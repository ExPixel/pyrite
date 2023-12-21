mod cli;
mod gba_runner;
mod ui;

use anyhow::Context as _;
use clap::Parser;
use cli::PyriteCli;
use eframe::Renderer;
use gba_runner::SharedGba;
mod config;
mod logging;

fn main() -> anyhow::Result<()> {
    let cli = PyriteCli::parse();
    let mut config = config::load().context("error while loading config")?;
    logging::init(&mut config).context("error while initializing logging")?;

    let renderer = if let Some(ref renderer) = config.gui.renderer {
        if renderer.eq_ignore_ascii_case("glow") || renderer.eq_ignore_ascii_case("gl") {
            #[cfg(feature = "glow")]
            {
                Renderer::Glow
            }

            #[cfg(not(feature = "glow"))]
            {
                tracing::error!("requested glow gui renderer was not compiled, using fallback");
                Renderer::default()
            }
        } else if renderer.eq_ignore_ascii_case("wgpu") {
            #[cfg(feature = "wgpu")]
            {
                Renderer::Wgpu
            }

            #[cfg(not(feature = "wgpu"))]
            {
                tracing::error!("requested wgpu gui renderer was not compiled, using fallback");
                Renderer::default()
            }
        } else {
            anyhow::bail!("unknown gui renderer in config: {renderer:?}");
        }
    } else {
        Renderer::default()
    };

    let native_options = eframe::NativeOptions {
        renderer,
        ..Default::default()
    };

    eframe::run_native(
        "Pyrite",
        native_options,
        Box::new(
            move |context| match ui::App::new(cli, config, SharedGba::new(), context) {
                Ok(app) => Box::new(app),
                Err(err) => {
                    tracing::error!(error = debug(err), "error while initializing app");
                    Box::new(AutocloseApp)
                }
            },
        ),
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

pub struct AutocloseApp;

impl eframe::App for AutocloseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}
