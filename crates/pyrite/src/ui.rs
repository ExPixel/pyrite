mod app_window;
mod gba_cpu_panel;
mod gba_image;
mod profiler;

use std::sync::Arc;

use crate::{
    cli::PyriteCli,
    config::{self, Config},
    gba_runner::SharedGba,
};
use ahash::HashSet;
use anyhow::Context as _;
use egui::{Ui, Vec2, ViewportId};
use parking_lot::{Mutex, MutexGuard};

use self::{
    app_window::{AppWindow, AppWindowCategory, AppWindowWrapper},
    gba_image::GbaImage,
    profiler::ProfilerWindow,
};

pub struct App {
    config: Config,
    screen: GbaImage,
    windows: Vec<app_window::AppWindowWrapper>,
    windows_visible: Arc<Mutex<HashSet<ViewportId>>>,
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

        let windows_visible = Arc::new(Mutex::new(HashSet::default()));
        #[cfg(feature = "profiling")]
        let profiler_window = ProfilerWindow::wrapped(windows_visible.clone(), context.storage);
        let windows = vec![
            #[cfg(feature = "profiling")]
            profiler_window,
            EguiSettingsWindow::wrapped(windows_visible.clone()),
            EguiInspectionWindow::wrapped(windows_visible.clone()),
            EguiTextureWindow::wrapped(windows_visible.clone()),
            EguiMemoryWindow::wrapped(windows_visible.clone()),
            EguiStyleWindow::wrapped(windows_visible.clone()),
        ];

        Ok(Self {
            config,
            screen,
            windows,
            windows_visible,
        })
    }

    fn render_menu(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| if ui.button("Open ROM...").clicked() {});
            ui.menu_button("View", |ui| {
                let categories = [
                    ("GBA", app_window::AppWindowCategory::Gba),
                    ("Egui", app_window::AppWindowCategory::Egui),
                ];

                let mut windows_visible = self.windows_visible.lock();
                for (category_name, category) in categories.into_iter() {
                    ui.menu_button(category_name, |ui| {
                        for window in self.windows.iter() {
                            if window.category() == category {
                                let mut display = window.visible_fast(&windows_visible);
                                let clicked = ui.checkbox(&mut display, window.title()).clicked();
                                if clicked {
                                    MutexGuard::unlocked(&mut windows_visible, || {
                                        window.set_visibility(display);
                                    });
                                    ui.close_menu();
                                    break;
                                }
                            }
                        }
                    });
                }
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

        let mut windows_visible = self.windows_visible.lock();
        for window in self.windows.iter() {
            if !windows_visible.contains(&window.viewport_id()) {
                continue;
            }
            MutexGuard::unlocked(&mut windows_visible, || {
                window.show_viewport_deferred(ctx);
            });
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        tracing::debug!("writing config file");

        for window in self.windows.iter() {
            window.save(storage);
        }

        if let Err(err) = config::store(&self.config).context("error while writing config file") {
            tracing::error!(error = debug(err), "error while saving");
        }
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.screen.destroy(gl);
    }
}

#[derive(Default)]
pub struct EguiSettingsWindow;

impl EguiSettingsWindow {
    pub fn wrapped(windows: Arc<Mutex<HashSet<ViewportId>>>) -> app_window::AppWindowWrapper {
        AppWindowWrapper::new_default::<Self>(windows)
    }
}

impl AppWindow for EguiSettingsWindow {
    type State = Self;

    fn ui(_state: &mut Self::State, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Egui Settings");
            ctx.settings_ui(ui);
        });
    }

    fn title() -> String {
        "Egui Settings".to_owned()
    }

    fn viewport_id() -> ViewportId {
        ViewportId::from_hash_of("egui_settings")
    }

    fn category() -> AppWindowCategory {
        AppWindowCategory::Egui
    }
}

#[derive(Default)]
pub struct EguiInspectionWindow;

impl EguiInspectionWindow {
    pub fn wrapped(windows: Arc<Mutex<HashSet<ViewportId>>>) -> app_window::AppWindowWrapper {
        AppWindowWrapper::new_default::<Self>(windows)
    }
}

impl AppWindow for EguiInspectionWindow {
    type State = Self;

    fn ui(_state: &mut Self::State, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Egui Inspection");
            ctx.inspection_ui(ui);
        });
    }

    fn title() -> String {
        "Egui Inspection".to_owned()
    }

    fn viewport_id() -> ViewportId {
        ViewportId::from_hash_of("egui_inspection")
    }

    fn category() -> AppWindowCategory {
        AppWindowCategory::Egui
    }
}

#[derive(Default)]
pub struct EguiTextureWindow;

impl EguiTextureWindow {
    pub fn wrapped(windows: Arc<Mutex<HashSet<ViewportId>>>) -> app_window::AppWindowWrapper {
        AppWindowWrapper::new_default::<Self>(windows)
    }
}

impl AppWindow for EguiTextureWindow {
    type State = Self;

    fn ui(_state: &mut Self::State, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Egui Textures");
            ctx.texture_ui(ui);
        });
    }

    fn title() -> String {
        "Egui Textures".to_owned()
    }

    fn viewport_id() -> ViewportId {
        ViewportId::from_hash_of("egui_textures")
    }

    fn category() -> AppWindowCategory {
        AppWindowCategory::Egui
    }
}

#[derive(Default)]
pub struct EguiMemoryWindow;

impl EguiMemoryWindow {
    pub fn wrapped(windows: Arc<Mutex<HashSet<ViewportId>>>) -> app_window::AppWindowWrapper {
        AppWindowWrapper::new_default::<Self>(windows)
    }
}

impl AppWindow for EguiMemoryWindow {
    type State = Self;

    fn ui(_state: &mut Self::State, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Egui Memory");
            ctx.memory_ui(ui);
        });
    }

    fn title() -> String {
        "Egui Memory".to_owned()
    }

    fn viewport_id() -> ViewportId {
        ViewportId::from_hash_of("egui_memory")
    }

    fn category() -> AppWindowCategory {
        AppWindowCategory::Egui
    }
}

#[derive(Default)]
pub struct EguiStyleWindow;

impl EguiStyleWindow {
    pub fn wrapped(windows: Arc<Mutex<HashSet<ViewportId>>>) -> app_window::AppWindowWrapper {
        AppWindowWrapper::new_default::<Self>(windows)
    }
}

impl AppWindow for EguiStyleWindow {
    type State = Self;

    fn ui(_state: &mut Self::State, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Egui Style");
            ctx.style_ui(ui);
        });
    }

    fn title() -> String {
        "Egui Style".to_owned()
    }

    fn viewport_id() -> ViewportId {
        ViewportId::from_hash_of("egui_style")
    }

    fn category() -> AppWindowCategory {
        AppWindowCategory::Egui
    }
}
