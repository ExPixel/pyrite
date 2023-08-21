mod cli;
mod gba_runner;
mod renderer;
mod worker;

use anyhow::Context as _;
use clap::Parser;
use cli::PyriteCli;
use gba_runner::SharedGba;

use crate::renderer::Renderer;
mod config;
mod logging;

fn main() -> anyhow::Result<()> {
    let cli = PyriteCli::parse();
    let mut config = config::load().context("error while loading config")?;
    logging::init(&mut config).context("error while initializing logging")?;

    #[cfg(feature = "tracy")]
    let _client = {
        tracing::debug!("starting tracy client");
        tracy_client::Client::start()
    };

    let renderer = config
        .read()
        .gui
        .renderer
        .as_ref()
        .map(|r| {
            if r.eq_ignore_ascii_case("glow") {
                Renderer::Glow
            } else if r.eq_ignore_ascii_case("wgpu") {
                Renderer::Wgpu
            } else {
                tracing::error!("unknown gui renderer in config: {r:?}");
                Renderer::Auto
            }
        })
        .unwrap_or(Renderer::Auto);
    let gba = SharedGba::new();

    if let Some(ref rom_path) = cli.rom {
        let rom = std::fs::read(rom_path)
            .with_context(|| format!("error while reading rom from {rom_path:?}"))?;
        gba.with_mut(move |g| {
            g.gba.set_gamepak(rom);
            g.gba.reset();
        });
        gba.unpause();
    }

    renderer::run(config, renderer, gba)
}
