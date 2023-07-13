use anyhow::Context as _;
use config::Config;
mod config;
mod logging;

fn main() -> anyhow::Result<()> {
    let mut config = config::load().context("error while loading config")?;
    logging::init(&mut config).context("error while initializing logging")?;

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Pyrite",
        native_options,
        Box::new(move |context| Box::new(App::new(config, context))),
    )
    .map_err(|current| {
        let mut ret_err = anyhow::Error::msg(current.to_string());
        let mut current: &dyn std::error::Error = &current;
        while let Some(cause) = current.source() {
            ret_err = anyhow::Error::msg(cause.to_string()).context(ret_err);
            current = cause;
        }
        ret_err
    })
    .context("error while running egui")?;

    Ok(())
}

pub struct App {
    config: Config,
}

impl App {
    pub fn new(config: Config, _context: &eframe::CreationContext<'_>) -> Self {
        Self { config }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        tracing::debug!("writing config file");
        if let Err(err) = config::store(&self.config).context("error while writing config file") {
            tracing::error!(error = debug(err), "error while saving");
        }
    }
}
