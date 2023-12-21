#[cfg(feature = "glow")]
pub mod glow;

#[cfg(feature = "wgpu")]
pub mod wgpu;

use crate::gba_runner::SharedGba;

#[cfg(feature = "glow")]
use self::glow::GbaImageGlow;
#[cfg(feature = "wgpu")]
use self::wgpu::GbaImageWgpu;

pub enum GbaImage {
    #[cfg(feature = "glow")]
    Glow(GbaImageGlow),

    #[cfg(feature = "wgpu")]
    Wgpu(GbaImageWgpu),
}

impl GbaImage {
    #[cfg(feature = "glow")]
    pub fn new_glow(gba: SharedGba) -> anyhow::Result<Self> {
        GbaImageGlow::new(gba).map(Self::Glow)
    }

    #[cfg(feature = "wgpu")]
    pub fn new_wgpu(gba: SharedGba) -> anyhow::Result<Self> {
        GbaImageWgpu::new(gba).map(Self::Wgpu)
    }

    pub fn paint(&mut self, rect: egui::Rect) -> egui::PaintCallback {
        match self {
            #[cfg(feature = "glow")]
            Self::Glow(glow) => glow.paint(rect),

            #[cfg(feature = "wgpu")]
            Self::Wgpu(wgpu) => wgpu.paint(rect),
        }
    }

    pub fn destroy(&mut self, gl: Option<&eframe::glow::Context>) {
        match self {
            #[cfg(feature = "glow")]
            Self::Glow(glow) => {
                if let Some(gl) = gl {
                    glow.destroy(gl);
                }
            }

            #[cfg(feature = "wgpu")]
            Self::Wgpu(wgpu) => wgpu.destroy(),
        }
    }
}
