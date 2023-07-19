use std::path::PathBuf;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use tracing::Level;

use crate::logging::LoggingReloadHandle;

impl Default for Config {
    fn default() -> Self {
        Config {
            gui: GuiConfig {
                renderer: Some("glow".into()),
            },

            logging: LoggingConfig {
                general: Some("debug".into()),
                gba: Some("debug".into()),
                arm: Some("debug".into()),
                wgpu: Some("info".into()),
                egui: Some("info".into()),
                extra_filters: Vec::new(),
                reload_handle: None,
            },
        }
    }
}

impl Config {
    pub fn get_log_filters(&self) -> anyhow::Result<String> {
        use std::fmt::Write as _;

        let get_level = |maybe_filter: &Option<&str>| -> anyhow::Result<Level> {
            let level_string = maybe_filter.as_ref().unwrap();
            level_string
                .parse()
                .with_context(|| format!("error while parsing log level `{level_string}`"))
        };

        let mut filters = if self.logging.general.is_some() {
            let level = get_level(&self.logging.general.as_deref())
                .context("error parsing general log level")?;
            level.to_string()
        } else {
            String::from("warn")
        };

        if self.logging.arm.is_some() {
            let level =
                get_level(&self.logging.arm.as_deref()).context("error parsing arm log level")?;
            write!(filters, ",arm={level}").unwrap();
        }

        if self.logging.gba.is_some() {
            let level =
                get_level(&self.logging.gba.as_deref()).context("error parsing gba log level")?;
            write!(filters, ",gba={level}").unwrap();
        }

        if self.logging.egui.is_some() {
            let level =
                get_level(&self.logging.egui.as_deref()).context("error parsing gba log level")?;
            write!(filters, ",eframe={level},egui_winit={level}").unwrap();
        }

        if self.logging.wgpu.is_some() {
            let level =
                get_level(&self.logging.wgpu.as_deref()).context("error parsing gba log level")?;
            write!(filters, ",wgpu_core={level},wgpu_hal={level},naga={level}").unwrap();
        }

        for extra in self.logging.extra_filters.iter() {
            write!(filters, ",{extra}").unwrap();
        }

        Ok(filters)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub gui: GuiConfig,
    pub logging: LoggingConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GuiConfig {
    pub renderer: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub general: Option<String>,
    pub gba: Option<String>,
    pub arm: Option<String>,
    pub egui: Option<String>,
    pub wgpu: Option<String>,

    #[serde(default)]
    pub extra_filters: Vec<String>,

    #[serde(skip)]
    pub reload_handle: Option<LoggingReloadHandle>,
}

fn get_config_path() -> anyhow::Result<PathBuf> {
    let config_dir = dirs::config_dir();
    let config_dir = if let Some(config_dir) = config_dir {
        let config_dir = config_dir.join("pyrite");
        std::fs::create_dir_all(&config_dir)
            .with_context(|| "error while creating config directory (path: {config_dir:?})")?;
        config_dir
    } else {
        std::env::current_dir().context("error while getting current directory")?
    };
    Ok(config_dir.join("pyrite.json"))
}

pub fn load() -> anyhow::Result<Config> {
    let config_path = get_config_path().context("error while getting config directory")?;

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let config_contents = std::fs::read_to_string(&config_path)
        .with_context(|| format!("error while reading config contents (path: {config_path:?})"))?;
    let config = serde_json::from_str(&config_contents)
        .with_context(|| "error while parsing config (path: {config_path:?})")?;
    Ok(config)
}

pub fn store(config: &Config) -> anyhow::Result<()> {
    let config_path = get_config_path().context("error while getting config directory")?;
    let mut config_file = std::fs::File::create(config_path)
        .with_context(|| "error while opening config file (path: {config_path:?})")?;
    serde_json::to_writer_pretty(&mut config_file, config)
        .with_context(|| "error while writing config (path: {config_path:?})")?;
    Ok(())
}
