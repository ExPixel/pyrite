use crate::video::GbaVideo;

pub struct GbaMemoryMappedHardware {
    pub video: Box<GbaVideo>,
}

impl GbaMemoryMappedHardware {
    pub fn new() -> Self {
        Self {
            video: Box::new(GbaVideo::new()),
        }
    }
}

impl Default for GbaMemoryMappedHardware {
    fn default() -> Self {
        Self::new()
    }
}
