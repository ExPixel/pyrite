mod gba_cpu_panel;
mod gba_image;

use crate::{
    config::{self, Config},
    gba_runner::SharedGba,
};
use anyhow::Context as _;
use egui::{epaint::Shadow, Rounding, Ui, Vec2};

use self::gba_image::GbaImage;

pub struct App {
    config: Config,
    gba: SharedGba,
    screen: GbaImage,
    main_content: TabContentType,
}

impl App {
    pub fn new(
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
            gba_data.request_repaint = Some(Box::new(move |_ready| {
                gba_egui_ctx.request_repaint();
            }));
        });

        let Some(screen) = screen else {
            anyhow::bail!("no renderer to construct screen texture");
        };

        gba.with_mut(|data| data.gba.reset());
        gba.unpause();

        Ok(Self {
            config,
            gba,
            screen,
            main_content: TabContentType::EmuGbaCpu,
        })
    }

    fn render_menu(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| if ui.button("Open ROM...").clicked() {});

            ui.menu_button("View", |ui| {
                ui.menu_button("GBA", |ui| {
                    for &view_item in TabContentType::emulator_views() {
                        let clicked = ui
                            .radio_value(&mut self.main_content, view_item, view_item.name())
                            .clicked();
                        if clicked {
                            ui.close_menu();
                            break;
                        }
                    }
                });

                ui.menu_button("Egui", |ui| {
                    for &view_item in TabContentType::egui_views() {
                        let clicked = ui
                            .radio_value(&mut self.main_content, view_item, view_item.name())
                            .clicked();
                        if clicked {
                            ui.close_menu();
                            break;
                        }
                    }
                });
            });
        });
    }

    fn render_right_panel(&mut self, ui: &mut Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            let screen_width = ui.available_width();
            let screen_height = (screen_width / 240.0) * 160.0;
            let (rect, _) = ui.allocate_exact_size(
                Vec2::new(screen_width as _, screen_height as _),
                egui::Sense::hover(),
            );
            let callback = egui::PaintCallback {
                rect,
                callback: self.screen.callback(),
            };
            ui.painter().add(callback);
        });
    }

    fn render_center_panel_tabs(&mut self, ui: &mut Ui) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            ui.horizontal(|ui| {
                for &view_item in TabContentType::emulator_views() {
                    let clicked = ui
                        .selectable_label(self.main_content == view_item, view_item.name())
                        .clicked();
                    if clicked {
                        self.main_content = view_item;
                        break;
                    }
                }
            });
        });
    }

    fn render_center_panel(&mut self, ui: &mut Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
            ui.add_space(ui.spacing().item_spacing.y);
            egui::containers::Frame::side_top_panel(ui.style()).show(ui, |ui| {
                self.render_center_panel_tabs(ui);
            });
            ui.separator();
            egui::ScrollArea::both().show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    match self.main_content {
                        TabContentType::EmuGbaCpu => gba_cpu_panel::render(ui, &self.gba),

                        TabContentType::EguiSettingsUi => ui.ctx().clone().settings_ui(ui),
                        TabContentType::EguiInspectionUi => ui.ctx().clone().inspection_ui(ui),
                        TabContentType::EguiTextureUi => ui.ctx().clone().texture_ui(ui),
                        TabContentType::EguiMemoryUi => ui.ctx().clone().memory_ui(ui),
                        TabContentType::EguiStyleUi => ui.ctx().clone().style_ui(ui),
                    }
                });
            })
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar_panel").show(ctx, |ui| self.render_menu(ui));
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(240.0)
            .width_range(240.0..=480.0)
            .frame(
                egui::containers::Frame::central_panel(&ctx.style())
                    .inner_margin(Vec2::new(0.0, 0.0))
                    .outer_margin(Vec2::new(0.0, 0.0))
                    .rounding(Rounding::none())
                    .shadow(Shadow::NONE),
            )
            .show(ctx, |ui| self.render_right_panel(ui));
        egui::CentralPanel::default()
            .frame(
                egui::containers::Frame::central_panel(&ctx.style())
                    .inner_margin(Vec2::new(0.0, 0.0))
                    .outer_margin(Vec2::new(0.0, 0.0))
                    .rounding(Rounding::none())
                    .shadow(Shadow::NONE),
            )
            .show(ctx, |ui| self.render_center_panel(ui));
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        tracing::debug!("writing config file");
        if let Err(err) = config::store(&self.config).context("error while writing config file") {
            tracing::error!(error = debug(err), "error while saving");
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum TabContentType {
    EmuGbaCpu,

    EguiSettingsUi,
    EguiInspectionUi,
    EguiTextureUi,
    EguiMemoryUi,
    EguiStyleUi,
}

impl TabContentType {
    pub fn emulator_views() -> &'static [TabContentType] {
        &[TabContentType::EmuGbaCpu]
    }

    pub fn egui_views() -> &'static [TabContentType] {
        &[
            TabContentType::EguiSettingsUi,
            TabContentType::EguiInspectionUi,
            TabContentType::EguiTextureUi,
            TabContentType::EguiMemoryUi,
            TabContentType::EguiStyleUi,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            TabContentType::EmuGbaCpu => "CPU",
            TabContentType::EguiSettingsUi => "Egui Settings",
            TabContentType::EguiInspectionUi => "Egui Inspection",
            TabContentType::EguiTextureUi => "Egui Textures",
            TabContentType::EguiMemoryUi => "Egui Memory",
            TabContentType::EguiStyleUi => "Egui Style",
        }
    }
}
