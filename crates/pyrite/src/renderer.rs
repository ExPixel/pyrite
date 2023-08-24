use crate::{config::SharedConfig, gba_runner::SharedGba};

#[cfg(feature = "glow")]
mod glow_renderer;

#[cfg(feature = "wgpu")]
mod wgpu_renderer;

mod common;

#[derive(Debug, Copy, Clone)]
pub enum Renderer {
    Auto,
    Wgpu,
    Glow,
}

impl Renderer {
    pub fn fallback() -> Option<Renderer> {
        if cfg!(feature = "wgpu") {
            Some(Self::Wgpu)
        } else if cfg!(feature = "glow") {
            Some(Self::Glow)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Renderer::Wgpu => write!(f, "wgpu"),
            Renderer::Glow => write!(f, "glow"),
            Renderer::Auto => write!(f, "auto"),
        }
    }
}

pub fn run(config: SharedConfig, renderer: Renderer, gba: SharedGba) -> anyhow::Result<()> {
    match renderer {
        #[cfg(feature = "backend-wgpu")]
        Renderer::Wgpu => common::run::<wgpu_renderer::WgpuApplication>(config, gba),

        #[cfg(feature = "backend-gl")]
        Renderer::Glow => common::run::<glow_renderer::GlowApplication>(config, gba),

        #[allow(unreachable_patterns)]
        _ => {
            if let Some(fallback) = Renderer::fallback() {
                tracing::debug!("{renderer} renderer not available, using fallback {fallback}");
                run(config, fallback, gba)
            } else {
                anyhow::bail!("{renderer} renderer not vailable, no fallback")
            }
        }
    }
}
