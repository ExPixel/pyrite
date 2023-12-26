use std::sync::Arc;

use eframe::Storage;
use egui::Ui;
use parking_lot::Mutex;
use puffin::GlobalFrameView;
use puffin_egui::ProfilerUi;

use super::app_window::{AppWindow, AppWindowCategory, AppWindowWrapper};

pub struct ProfilerWindow {
    profiler: Profiler,
}

impl ProfilerWindow {
    fn new(storage: Option<&dyn eframe::Storage>) -> Self {
        Self {
            profiler: Profiler::new(storage),
        }
    }

    pub fn wrapped(
        windows: Arc<Mutex<egui::ahash::HashSet<egui::ViewportId>>>,
        storage: Option<&dyn eframe::Storage>,
    ) -> AppWindowWrapper {
        AppWindowWrapper::new::<Self>(windows, Self::new(storage))
    }
}

impl AppWindow for ProfilerWindow {
    type State = Self;

    fn ui(state: &mut Self::State, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Profiler");
            render(ui, &mut state.profiler);
        });
    }

    fn title() -> String {
        "Profiler".to_owned()
    }

    fn viewport_id() -> egui::ViewportId {
        egui::ViewportId::from_hash_of("profiler")
    }

    fn category() -> AppWindowCategory {
        AppWindowCategory::Gba
    }

    fn save(state: &mut Self::State, storage: &mut dyn Storage) {
        state.profiler.save(storage);
    }
}

pub fn render(ui: &mut Ui, profiler: &mut Profiler) {
    #[cfg(feature = "puffin")]
    {
        let mut enabled = puffin::are_scopes_on();
        if ui.selectable_label(enabled, "Collect Frames").clicked() {
            enabled = !enabled;
        }

        puffin::set_scopes_on(false);
        profiler.profiler_ui.ui(
            ui,
            &mut puffin_egui::MaybeMutRef::MutRef(&mut profiler.frame_view.lock()),
        );
        puffin::set_scopes_on(enabled);
    }

    #[cfg(not(feature = "puffin"))]
    ui.label("profiling feature is not enabled (must be done at compile time)");
}

pub struct Profiler {
    profiler_ui: ProfilerUi,
    frame_view: GlobalFrameView,
}

impl Profiler {
    pub fn new(storage: Option<&dyn eframe::Storage>) -> Self {
        let profiler_ui =
            if let Some(profiler) = storage.and_then(|storage| storage.get_string("profiler")) {
                match serde_json::from_str(&profiler) {
                    Ok(profiler) => profiler,
                    Err(err) => {
                        let err = anyhow::Error::from(err);
                        tracing::error!(error = debug(err), "error deserializing profiler");
                        ProfilerUi::default()
                    }
                }
            } else {
                ProfilerUi::default()
            };
        Profiler {
            profiler_ui,
            frame_view: GlobalFrameView::default(),
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) {
        match serde_json::to_string(&self.profiler_ui) {
            Ok(json) => storage.set_string("profiler", json),
            Err(err) => {
                let err = anyhow::Error::from(err);
                tracing::error!(error = debug(err), "error serializing profiler");
            }
        }
    }
}
