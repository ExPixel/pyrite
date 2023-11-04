use anyhow::Context as _;
use crossbeam::channel::{self, Receiver, Sender};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    prelude::{Backend, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};
use std::{io, sync::Arc};

pub fn run() -> DebuggerHandle {
    let (handle_send, handle_recv) = channel::unbounded();
    let (debugger_send, debugger_recv) = channel::unbounded();

    let debugger_send_event = debugger_send.clone();

    std::thread::spawn(move || {
        std::thread::scope(move |scope| {
            scope.spawn(|| {
                if let Err(err) = run_debugger_inner(handle_send, debugger_recv) {
                    tracing::error!(error = debug(err), "error while running TUI");
                }
            });

            scope.spawn(move || loop {
                match event::read().context("error while reading terminal event") {
                    Ok(event) => {
                        if debugger_send_event
                            .send(DebuggerEvent::TerminalEvent(event))
                            .is_err()
                        {
                            tracing::debug!("cannot send terminal event, debugger has shutdown");
                            break;
                        }
                    }
                    Err(err) => {
                        tracing::error!(error = debug(err), "error while waiting for TUI events");
                        break;
                    }
                }
            });
        })
    });

    DebuggerHandle {
        inner: Arc::new(DebuggerHandleInner {
            sender: debugger_send,
            receiver: handle_recv,
        }),
    }
}

#[derive(Clone)]
pub struct DebuggerHandle {
    inner: Arc<DebuggerHandleInner>,
}

impl DebuggerHandle {
    pub fn stop(&self, wait: bool) {
        tracing::debug!("sending debugger stop event");
        let _ = self.inner.sender.send(DebuggerEvent::Stop);
        if wait {
            let _ = self.inner.receiver.recv();
            tracing::debug!("debugger stopped");
        }
    }
}

pub struct DebuggerHandleInner {
    sender: Sender<DebuggerEvent>,
    receiver: Receiver<()>,
}

struct Debugger {
    current_tab: DebuggerTab,
}

impl Debugger {
    fn new() -> Self {
        Self {
            current_tab: DebuggerTab::Logs,
        }
    }

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());
        self.draw_tabs(f, chunks[0]);
    }

    fn draw_tabs<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) {
        let titles = DebuggerTab::ALL
            .into_iter()
            .map(|t| Line::from(Span::styled(t.name(), Style::default().fg(Color::Green))))
            .collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(self.current_tab.index());
        f.render_widget(tabs, rect);
    }

    fn on_terminal_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let index = self.current_tab.index();
                        if index < DebuggerTab::ALL.len() - 1 {
                            self.current_tab = DebuggerTab::ALL[index + 1];
                        }
                    }

                    KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let index = self.current_tab.index();
                        if index > 0 {
                            self.current_tab = DebuggerTab::ALL[index - 1];
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
#[repr(usize)]
enum DebuggerTab {
    Logs = 0,
    Disassembly,
    Memory,
}

impl DebuggerTab {
    const ALL: [DebuggerTab; 3] = [
        DebuggerTab::Logs,
        DebuggerTab::Disassembly,
        DebuggerTab::Memory,
    ];

    fn name(self) -> &'static str {
        match self {
            DebuggerTab::Logs => "Logs",
            DebuggerTab::Disassembly => "Disassembly",
            DebuggerTab::Memory => "Memory",
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

fn run_debugger_inner(
    _handle_send: Sender<()>,
    debugger_recv: Receiver<DebuggerEvent>,
) -> anyhow::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut debugger = Debugger::new();

    loop {
        if let Err(err) = terminal.draw(|f| debugger.draw(f)) {
            tracing::error!(error = debug(err), "debugger TUI error");
        }

        match debugger_recv.recv() {
            Ok(DebuggerEvent::TerminalEvent(event)) => {
                debugger.on_terminal_event(event);
            }
            Ok(DebuggerEvent::Stop) | Err(_) => break,
        }
    }

    tracing::info!("shutting down debugger");

    // restore terminal
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
enum DebuggerEvent {
    TerminalEvent(Event),
    Stop,
}
