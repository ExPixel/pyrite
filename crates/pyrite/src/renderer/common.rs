use crate::{config::SharedConfig, gba_runner::SharedGba};
use ahash::AHashMap;
use anyhow::Context as _;
use winit::{
    dpi::LogicalSize,
    event::{Event, ModifiersState, ScanCode, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::Window,
};

pub fn run<A>(config: SharedConfig, gba: SharedGba) -> anyhow::Result<()>
where
    A: Application,
    A::Resources: 'static,
{
    let event_loop = EventLoop::new();

    let init_context = AppInitContext {
        event_loop: &event_loop,
        config: &config,
        gba: &gba,
    };
    let mut resources =
        A::init(init_context).context("error while initializing application resources")?;
    let mut keyboard = KeyboardState::new();

    event_loop.run(move |event, event_loop_window_target, control_flow| {
        let mut event_context = AppEventContext {
            gba: &gba,
            config: &config,
            resources: &mut resources,
            event,
            event_loop_window_target,
            control_flow,
            keyboard: &mut keyboard,
        };

        if let Err(err) = self::process_event(&mut event_context) {
            tracing::error!(error = debug(err), "error in common event processing");
            control_flow.set_exit_with_code(1);
        } else if let Err(err) = A::handle_event(event_context) {
            tracing::error!(error = debug(err), "error in application event processing");
            control_flow.set_exit_with_code(1);
        } else {
            keyboard.transition_keys();
        }

        if matches!(
            control_flow,
            &mut ControlFlow::Exit | &mut ControlFlow::ExitWithCode(_)
        ) {
            gba.with_mut(|gba| {
                if let Some(debugger) = gba.debugger.take() {
                    debugger.stop(true);
                }
            });
        }
    });
}

pub(crate) fn process_event<R: ResourcesCommon>(
    context: &mut AppEventContext<R>,
) -> anyhow::Result<()> {
    let AppEventContext {
        config,
        event,
        control_flow,
        keyboard,
        resources,
        ..
    } = context;

    match event {
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } if size.width != 0 && size.height != 0 => {
            {
                let mut config = config.write();
                config.gui.window_width = Some(size.width);
                config.gui.window_height = Some(size.height);
            }
            crate::config::schedule_store(config);
        }

        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            control_flow.set_exit();
        }

        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => match input.state {
            winit::event::ElementState::Pressed => {
                keyboard.on_key_down(input.scancode, input.virtual_keycode)
            }
            winit::event::ElementState::Released => {
                keyboard.on_key_up(input.scancode, input.virtual_keycode)
            }
        },

        Event::WindowEvent {
            event: WindowEvent::ModifiersChanged(modifiers),
            ..
        } => keyboard.modifiers = *modifiers,

        _ => {}
    }

    let resize_keys = [
        Key::Virtual(VirtualKeyCode::Key1),
        Key::Virtual(VirtualKeyCode::Key2),
        Key::Virtual(VirtualKeyCode::Key3),
    ];
    for (idx, key) in resize_keys.into_iter().enumerate() {
        if !keyboard.shift() || !keyboard.ctrl() || !keyboard.pressed(key) {
            continue;
        }
        if let Some(window) = resources.window() {
            let mul = idx as u32 + 1;
            window.set_inner_size(LogicalSize::new(240 * mul, 160 * mul));
        }
    }

    Ok(())
}

pub trait Application {
    type Resources: ResourcesCommon;

    fn init(context: AppInitContext) -> anyhow::Result<Self::Resources>;
    fn handle_event(context: AppEventContext<Self::Resources>) -> anyhow::Result<()>;
}

pub struct AppInitContext<'a> {
    pub event_loop: &'a EventLoop<()>,
    pub config: &'a SharedConfig,
    pub gba: &'a SharedGba,
}

pub struct AppEventContext<'a, 'e, R> {
    pub gba: &'a SharedGba,
    pub config: &'a SharedConfig,
    pub resources: &'a mut R,
    pub event: Event<'e, ()>,
    pub event_loop_window_target: &'a EventLoopWindowTarget<()>,
    pub control_flow: &'a mut ControlFlow,
    pub keyboard: &'a mut KeyboardState,
}

pub struct KeyboardState {
    scan_code_state: AHashMap<ScanCode, KeyState>,
    vkey_code_state: AHashMap<VirtualKeyCode, KeyState>,
    modifiers: ModifiersState,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            scan_code_state: AHashMap::new(),
            vkey_code_state: AHashMap::new(),
            modifiers: ModifiersState::default(),
        }
    }

    pub fn is_up(&self, key: Key) -> bool {
        matches!(self.get_key_state(key), KeyState::Up | KeyState::Released)
    }

    pub fn is_down(&self, key: Key) -> bool {
        matches!(self.get_key_state(key), KeyState::Down | KeyState::Pressed)
    }

    pub fn pressed(&self, key: Key) -> bool {
        matches!(self.get_key_state(key), KeyState::Pressed)
    }

    pub fn released(&self, key: Key) -> bool {
        matches!(self.get_key_state(key), KeyState::Released)
    }

    fn get_key_state(&self, key: Key) -> KeyState {
        match key {
            Key::Virtual(code) => self
                .vkey_code_state
                .get(&code)
                .copied()
                .unwrap_or(KeyState::Up),
            Key::ScanCode(code) => self
                .scan_code_state
                .get(&code)
                .copied()
                .unwrap_or(KeyState::Up),
        }
    }

    fn on_key_down(&mut self, scan_code: ScanCode, vkey_code: Option<VirtualKeyCode>) {
        let entry = self
            .scan_code_state
            .entry(scan_code)
            .or_insert(KeyState::Up);

        *entry = if matches!(*entry, KeyState::Up | KeyState::Released) {
            KeyState::Pressed
        } else {
            KeyState::Down
        };

        if let Some(vkey_code) = vkey_code {
            let entry = self
                .vkey_code_state
                .entry(vkey_code)
                .or_insert(KeyState::Up);

            *entry = if matches!(*entry, KeyState::Up | KeyState::Released) {
                KeyState::Pressed
            } else {
                KeyState::Down
            };
        }
    }

    fn on_key_up(&mut self, scan_code: ScanCode, vkey_code: Option<VirtualKeyCode>) {
        let entry = self
            .scan_code_state
            .entry(scan_code)
            .or_insert(KeyState::Up);

        *entry = if matches!(*entry, KeyState::Down | KeyState::Pressed) {
            KeyState::Released
        } else {
            KeyState::Up
        };

        if let Some(vkey_code) = vkey_code {
            let entry = self
                .vkey_code_state
                .entry(vkey_code)
                .or_insert(KeyState::Up);

            *entry = if matches!(*entry, KeyState::Down | KeyState::Pressed) {
                KeyState::Released
            } else {
                KeyState::Up
            };
        }
    }

    fn transition_keys(&mut self) {
        for state in self.scan_code_state.values_mut() {
            match *state {
                KeyState::Pressed => *state = KeyState::Down,
                KeyState::Released => *state = KeyState::Up,
                _ => {}
            }
        }

        for state in self.vkey_code_state.values_mut() {
            match *state {
                KeyState::Pressed => *state = KeyState::Down,
                KeyState::Released => *state = KeyState::Up,
                _ => {}
            }
        }
    }

    pub fn shift(&self) -> bool {
        self.modifiers.shift()
    }

    pub fn ctrl(&self) -> bool {
        self.modifiers.ctrl()
    }

    pub fn alt(&self) -> bool {
        self.modifiers.alt()
    }

    pub fn logo(&self) -> bool {
        self.modifiers.logo()
    }
}

#[derive(Copy, Clone)]
pub enum Key {
    Virtual(VirtualKeyCode),
    ScanCode(u32),
}

#[derive(Copy, Clone)]
pub enum KeyState {
    Down,
    Pressed,
    Released,
    Up,
}

pub trait ResourcesCommon {
    fn window(&self) -> Option<&Window>;
}
