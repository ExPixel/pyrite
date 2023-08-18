use eframe::Storage;
use egui::Ui;
use puffin::GlobalFrameView;
use puffin_egui::ProfilerUi;

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
