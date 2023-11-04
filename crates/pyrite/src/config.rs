use std::{
    path::PathBuf,
    sync::{atomic, Arc},
    time::Duration,
};

use anyhow::Context as _;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::worker;

impl Default for Config {
    fn default() -> Self {
        Config {
            gui: GuiConfig {
                renderer: Some("glow".into()),
                ..Default::default()
            },

            logging: LoggingConfig {
                general: Some("debug".into()),
                gba: Some("debug".into()),
                arm: Some("debug".into()),
                graphics: Some("info".into()),
                ..Default::default()
            },

            path: None,
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

        if self.logging.graphics.is_some() {
            let level = get_level(&self.logging.graphics.as_deref())
                .context("error parsing gba log level")?;
            write!(filters, ",wgpu_core={level},wgpu_hal={level},naga={level}").unwrap();
            write!(filters, ",glow={level}").unwrap();
        }

        for extra in self.logging.extra_filters.iter() {
            write!(filters, ",{extra}").unwrap();
        }
        // println!("filters: {filters}");

        Ok(filters)
    }
}

#[derive(Default, Clone)]
pub struct SharedConfig {
    inner: Arc<RwLock<Config>>,
}

impl SharedConfig {
    pub fn read(&self) -> ConfigRead<'_> {
        ConfigRead {
            inner: self.inner.read(),
        }
    }

    pub fn write(&self) -> ConfigWrite<'_> {
        ConfigWrite {
            inner: self.inner.write(),
        }
    }
}

pub struct ConfigRead<'a> {
    inner: RwLockReadGuard<'a, Config>,
}

pub struct ConfigWrite<'a> {
    inner: RwLockWriteGuard<'a, Config>,
}

impl std::ops::Deref for ConfigRead<'_> {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::Deref for ConfigWrite<'_> {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for ConfigWrite<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub gui: GuiConfig,
    pub logging: LoggingConfig,

    #[serde(skip)]
    path: Option<PathBuf>,
}

#[derive(Serialize, Default, Deserialize)]
pub struct GuiConfig {
    pub renderer: Option<String>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub general: Option<String>,
    pub gba: Option<String>,
    pub arm: Option<String>,
    pub graphics: Option<String>,
    pub extra_filters: Vec<String>,

    #[serde(skip)]
    pub reload: Option<Box<dyn Send + Sync + 'static + Fn(EnvFilter) -> anyhow::Result<()>>>,
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

pub fn load() -> anyhow::Result<SharedConfig> {
    let config_path = get_config_path().context("error while getting config directory")?;

    if !config_path.exists() {
        return Ok(SharedConfig::default());
    }

    let config_contents = std::fs::read_to_string(&config_path)
        .with_context(|| format!("error while reading config contents (path: {config_path:?})"))?;
    let mut config: Config = serde_json::from_str(&config_contents)
        .with_context(|| "error while parsing config (path: {config_path:?})")?;
    config.path = Some(config_path);
    let inner = Arc::new(RwLock::new(config));
    Ok(SharedConfig { inner })
}

#[allow(dead_code)]
pub fn store(config: &SharedConfig) -> anyhow::Result<()> {
    let mut config = config.write();
    store_internal(&mut config)
}

fn store_internal(config: &mut Config) -> anyhow::Result<()> {
    if config.path.is_none() {
        let config_path = get_config_path().context("error while getting config directory")?;
        config.path = Some(config_path);
    }

    let mut config_file = std::fs::File::create(config.path.as_ref().unwrap())
        .with_context(|| "error while opening config file (path: {config_path:?})")?;
    serde_json::to_writer_pretty(&mut config_file, config)
        .with_context(|| "error while writing config (path: {config_path:?})")?;
    Ok(())
}

pub fn schedule_store(config: &SharedConfig) {
    static STORE_SCHEDULED: atomic::AtomicBool = atomic::AtomicBool::new(false);

    if STORE_SCHEDULED
        .compare_exchange(
            false,
            true,
            atomic::Ordering::Release,
            atomic::Ordering::Acquire,
        )
        .is_ok()
    {
        let config = config.clone();
        worker::spawn_in(
            move || {
                let mut config = config.write();
                if let Err(err) = self::store_internal(&mut config) {
                    tracing::error!(error = debug(err), "error while storing config (scheduled)");
                } else {
                    let path = config.path.as_ref().unwrap();
                    tracing::debug!("wrote config to {path:?}");
                }
                STORE_SCHEDULED.store(false, atomic::Ordering::Release);
            },
            Duration::from_secs(1),
        );
    }
}
