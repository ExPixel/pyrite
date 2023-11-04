use std::io;

use anyhow::Context;
use tracing::{instrument::WithSubscriber, metadata::LevelFilter};
use tracing_subscriber::{
    layer::Layered, prelude::__tracing_subscriber_SubscriberExt, reload::Handle, EnvFilter, Layer,
    Registry,
};

use crate::config::SharedConfig;

pub fn init(config: &mut SharedConfig, debugger_mode: bool) -> anyhow::Result<()> {
    let log_filters = config
        .read()
        .get_log_filters()
        .context("error while constructing log filters")?;
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .parse(log_filters)
        .context("parsing log filter")?;
    let (env_filter, reload_handle) = tracing_subscriber::reload::Layer::new(env_filter);
    config.write().logging.reload = Some(Box::new(move |filter| {
        reload_handle
            .modify(|f| *f = filter)
            .context("error while updating log filters")
    }));

    let stderr_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_writer(io::stderr);

    let layers = tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer);

    if debugger_mode {
    } else {
        tracing::subscriber::set_global_default(layers).expect("Unable to set a global collector");
    }

    Ok(())
}

#[allow(dead_code)]
pub fn reload(config: &SharedConfig) -> anyhow::Result<()> {
    let log_filters = config
        .read()
        .get_log_filters()
        .context("error while constructing log filters")?;
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .parse(log_filters)
        .context("parsing log filter")?;
    if let Some(ref reload_handle) = config.write().logging.reload {
        (reload_handle)(env_filter)
    } else {
        Err(anyhow::Error::msg("logging config reload handle not found"))
    }
}
