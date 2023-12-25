mod gba_cpu_panel;
mod gba_image;
mod profiler;

use std::{ops::DerefMut, sync::Arc};

use crate::{
    cli::PyriteCli,
    config::{self, Config},
    gba_runner::SharedGba,
};
use anyhow::Context as _;
use egui::{ahash::HashSet, Context, Ui, Vec2, ViewportBuilder, ViewportId};
use parking_lot::Mutex;

use self::{gba_image::GbaImage, profiler::Profiler};

pub struct App {
    config: Config,
    gba: SharedGba,
    screen: GbaImage,
    #[cfg(feature = "profiling")]
    profiler: Arc<Mutex<Profiler>>,
    windows: Arc<Mutex<HashSet<WindowType>>>,
}

pub struct AppTreeElements {
    gba: SharedGba,
}

impl App {
    pub fn new(
        cli: PyriteCli,
        config: Config,
        gba: SharedGba,
        context: &eframe::CreationContext<'_>,
    ) -> anyhow::Result<Self> {
        let mut screen: Option<GbaImage> = None;

        #[cfg(feature = "glow")]
        if context.gl.is_some() {
            let image = GbaImage::new_glow(gba.clone())
                .context("error while creating screen texture using glow")?;
            screen = Some(image);
        }

        #[cfg(feature = "wgpu")]
        if context.wgpu_render_state.is_some() {
            let image = GbaImage::new_wgpu(gba.clone())
                .context("error while creating screen texture using wgpu")?;
            screen = Some(image);
        }

        let gba_egui_ctx = context.egui_ctx.clone();
        gba.with_mut(move |gba_data| {
            gba_data.request_repaint = Some(Box::new(move |_ready, _| {
                gba_egui_ctx.request_repaint();
            }));
        });

        let Some(screen) = screen else {
            anyhow::bail!("no renderer to construct screen texture");
        };

        let rom = if let Some(path) = cli.rom {
            Some(std::fs::read(&path).with_context(|| format!("error reading ROM from {path:?}"))?)
        } else {
            None
        };

        gba.with_mut(|data| {
            if let Some(rom) = rom {
                data.gba.set_gamepak(rom);
            } else {
                data.gba.set_noop_gamepak();
            }

            data.gba.reset();
        });
        gba.unpause();

        Ok(Self {
            config,
            gba,
            screen,
            #[cfg(feature = "profiling")]
            profiler: Arc::new(Mutex::new(Profiler::new(context.storage))),
            windows: Arc::new(Mutex::new(HashSet::default())),
        })
    }

    fn render_menu(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| if ui.button("Open ROM...").clicked() {});
            ui.menu_button("View", |ui| {
                let window_menu_begin = |window: WindowType, set: &HashSet<WindowType>| -> bool {
                    set.contains(&window)
                };
                let window_menu_end =
                    |window: WindowType, set: &mut HashSet<WindowType>, display: bool| {
                        let contains = set.contains(&window);
                        if contains && !display {
                            set.remove(&window);
                        } else if !contains && display {
                            set.insert(window);
                        }
                    };
                let handle_windows =
                    |windows: &[WindowType], set: &mut HashSet<WindowType>, ui: &mut Ui| {
                        for &view_item in windows {
                            let mut display = window_menu_begin(view_item, set);
                            let clicked = ui.checkbox(&mut display, view_item.title()).clicked();
                            window_menu_end(view_item, set, display);
                            if clicked {
                                ui.close_menu();
                                break;
                            }
                        }
                    };

                ui.menu_button("GBA", |ui| {
                    handle_windows(WindowType::emulator_views(), &mut self.windows.lock(), ui);
                });

                ui.menu_button("Egui", |ui| {
                    handle_windows(WindowType::egui_views(), &mut self.windows.lock(), ui);
                });
            });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar_panel").show(ctx, |ui| self.render_menu(ui));
        egui::CentralPanel::default().show(ctx, |ui| {
            let screen_width = ui.available_width();
            let screen_height = (screen_width / 240.0) * 160.0;
            let (rect, _) = ui.allocate_exact_size(
                Vec2::new(screen_width as _, screen_height as _),
                egui::Sense::hover(),
            );
            ui.painter().add(self.screen.paint(rect));
        });

        let windows = self.windows.lock().clone();
        for &w in windows.iter() {
            match w {
                WindowType::EmuGbaDisassembly => todo!(),

                WindowType::EmuProfiler => {
                    let profiler = self.profiler.clone();
                    viewport_deferred_helper(ctx, w, self.windows.clone(), move |ctx, _| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            profiler::render(ui, &mut profiler.lock());
                        });
                    });
                }

                WindowType::EguiSettingsUi => {
                    viewport_deferred_helper(ctx, w, self.windows.clone(), move |ctx, ui| {
                        ctx.settings_ui(ui);
                    });
                }

                WindowType::EguiInspectionUi => {
                    viewport_deferred_helper(ctx, w, self.windows.clone(), move |ctx, ui| {
                        ctx.inspection_ui(ui);
                    });
                }

                WindowType::EguiTextureUi => {
                    viewport_deferred_helper(ctx, w, self.windows.clone(), move |ctx, ui| {
                        ctx.texture_ui(ui);
                    });
                }

                WindowType::EguiMemoryUi => {
                    viewport_deferred_helper(ctx, w, self.windows.clone(), move |ctx, ui| {
                        ctx.memory_ui(ui);
                    });
                }

                WindowType::EguiStyleUi => {
                    viewport_deferred_helper(ctx, w, self.windows.clone(), move |ctx, ui| {
                        ctx.style_ui(ui);
                    });
                }
            }
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        tracing::debug!("writing config file");
        self.profiler.lock().save(storage);
        if let Err(err) = config::store(&self.config).context("error while writing config file") {
            tracing::error!(error = debug(err), "error while saving");
        }
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.screen.destroy(gl);
    }
}

fn viewport_deferred_helper<F>(
    ctx: &Context,
    window_type: WindowType,
    windows: Arc<Mutex<HashSet<WindowType>>>,
    f: F,
) where
    F: 'static + Send + Sync + Fn(&Context, &mut Ui),
{
    ctx.show_viewport_deferred(window_type.id(), window_type.options(), move |ctx, _| {
        egui::CentralPanel::default().show(ctx, |ui| {
            f(ctx, ui);
        });
        ctx.input(|input| {
            if input.viewport().close_requested() {
                windows.lock().remove(&window_type);
            }
        });
    });
}

#[derive(PartialEq, Clone, Copy, Hash, Eq, Debug)]
enum WindowType {
    EmuGbaDisassembly,
    EmuProfiler,

    EguiSettingsUi,
    EguiInspectionUi,
    EguiTextureUi,
    EguiMemoryUi,
    EguiStyleUi,
}

impl WindowType {
    pub fn emulator_views() -> &'static [WindowType] {
        &[WindowType::EmuGbaDisassembly, WindowType::EmuProfiler]
    }

    pub fn tab_views() -> &'static [WindowType] {
        &[WindowType::EmuGbaDisassembly, WindowType::EmuProfiler]
    }

    pub fn egui_views() -> &'static [WindowType] {
        &[
            WindowType::EguiSettingsUi,
            WindowType::EguiInspectionUi,
            WindowType::EguiTextureUi,
            WindowType::EguiMemoryUi,
            WindowType::EguiStyleUi,
        ]
    }

    pub fn title(self) -> &'static str {
        match self {
            WindowType::EmuGbaDisassembly => "Disassembly",
            WindowType::EmuProfiler => "Profiler",
            WindowType::EguiSettingsUi => "Egui Settings",
            WindowType::EguiInspectionUi => "Egui Inspection",
            WindowType::EguiTextureUi => "Egui Textures",
            WindowType::EguiMemoryUi => "Egui Memory",
            WindowType::EguiStyleUi => "Egui Style",
        }
    }

    pub fn id(self) -> ViewportId {
        ViewportId::from_hash_of(self)
    }

    pub fn options(self) -> ViewportBuilder {
        ViewportBuilder {
            title: Some(self.title().into()),
            ..Default::default()
        }
    }
}
