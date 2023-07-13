use anyhow::Context;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{layer::Layered, reload::Handle, EnvFilter, Registry};

use crate::config::Config;

pub type LoggingReloadHandle =
    Handle<EnvFilter, Layered<tracing_subscriber::fmt::Layer<Registry>, Registry>>;

pub fn init(config: &mut Config) -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .parse(
            config
                .get_log_filters()
                .context("error while constructing log filters")?,
        )
        .context("parsing log filter")?;
    let builder = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .with_filter_reloading();
    config.logging.reload_handle = Some(builder.reload_handle());
    builder.init();
    Ok(())
}

#[allow(dead_code)]
pub fn reload(config: &Config) -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .parse(
            config
                .get_log_filters()
                .context("error while constructing log filters")?,
        )
        .context("parsing log filter")?;
    if let Some(ref reload_handle) = config.logging.reload_handle {
        reload_handle
            .modify(move |filter| *filter = env_filter)
            .context("error while modifying logging filters")
    } else {
        Err(anyhow::Error::msg("logging config reload handle not found"))
    }
}
