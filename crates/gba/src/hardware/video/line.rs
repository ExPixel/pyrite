use util::bits::BitOps;

use crate::{hardware::palette::Palette, memory::VRAM_SIZE};

use super::{registers::GbaVideoRegisters, HBlankContext, VISIBLE_LINE_WIDTH};

#[derive(Default)]
pub struct GbaLine {
    layers: [LayerLine; 5],
    objwin: LineBits,
    layer_attrs: [LayerAttrs; 5],
}

impl GbaLine {
    pub fn put(&mut self, layer: Layer, x: usize, pixel: impl Into<Pixel>) {
        self.layers[layer as usize].pixels[x] = pixel.into();
    }

    pub fn blend(&mut self, output: &mut [u16; VISIBLE_LINE_WIDTH], context: BlendContext) {
        #[cfg(feature = "profiling")]
        profiling::scope!("line::blend");

        for (x, output) in output.iter_mut().enumerate() {
            let pixel = self.layers[2].pixels[x];
            *output = pixel.value;
        }
    }
}

#[derive(Clone, Copy)]
pub struct BlendContext<'a> {
    pub registers: &'a GbaVideoRegisters,
    pub vram: &'a [u8; VRAM_SIZE],
    pub palette: &'a Palette,
}

impl<'a> BlendContext<'a> {
    pub fn with_hblank(registers: &'a GbaVideoRegisters, context: HBlankContext<'a>) -> Self {
        Self::new(registers, context.vram, context.palette)
    }

    pub fn new(
        registers: &'a GbaVideoRegisters,
        vram: &'a [u8; VRAM_SIZE],
        palette: &'a Palette,
    ) -> Self {
        Self {
            registers,
            vram,
            palette,
        }
    }
}

pub enum Layer {
    Bg0 = 0,
    Bg1 = 1,
    Bg2 = 2,
    Bg3 = 3,
    Obj = 4,
}

struct LayerLine {
    attrs: LayerAttrs,
    pixels: [Pixel; VISIBLE_LINE_WIDTH],
}

impl Default for LayerLine {
    fn default() -> Self {
        Self {
            pixels: [Pixel::default(); VISIBLE_LINE_WIDTH],
            attrs: LayerAttrs::default(),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Pixel {
    value: u16,
}

impl Pixel {
    pub const fn new(value: u16) -> Self {
        Self { value }
    }
}

impl From<ObjPixel8Bpp> for Pixel {
    fn from(value: ObjPixel8Bpp) -> Self {
        Self { value: value.0 }
    }
}

impl From<ObjPixel4Bpp> for Pixel {
    fn from(value: ObjPixel4Bpp) -> Self {
        Self { value: value.0 }
    }
}

#[derive(Default, Clone, Copy)]
pub struct ObjPixel8Bpp(u16);

impl ObjPixel8Bpp {
    pub fn new(attrs: PixelAttrs, entry: u8) -> Self {
        Self(attrs.value as u16 | ((entry as u16) << 8))
    }
}

impl From<Pixel> for ObjPixel8Bpp {
    fn from(pixel: Pixel) -> Self {
        Self(pixel.value)
    }
}

#[derive(Default, Clone, Copy)]
pub struct ObjPixel4Bpp(u16);

impl ObjPixel4Bpp {
    pub fn new(attrs: PixelAttrs, palette: u8, entry: u8) -> Self {
        Self((attrs.value as u16) | ((palette as u16) << 12) | ((entry as u16) << 8))
    }
}

impl From<Pixel> for ObjPixel4Bpp {
    fn from(pixel: Pixel) -> Self {
        Self(pixel.value)
    }
}

#[derive(Default, Clone, Copy)]
pub struct BgPixel8Bpp(u16);

impl BgPixel8Bpp {
    pub fn new(entry: u8) -> Self {
        Self(entry as u16)
    }

    pub fn entry(&self) -> u8 {
        self.0.get_bit_range(0..8) as u8
    }
}

impl From<Pixel> for BgPixel8Bpp {
    fn from(pixel: Pixel) -> Self {
        Self(pixel.value)
    }
}

#[derive(Default, Clone, Copy)]
pub struct BgPixel4Bpp(u16);

impl BgPixel4Bpp {
    pub fn new(palette: u8, entry: u8) -> Self {
        Self(((palette as u16) << 4) | (entry as u16))
    }
}

impl From<Pixel> for BgPixel4Bpp {
    fn from(pixel: Pixel) -> Self {
        Self(pixel.value)
    }
}

#[derive(Clone, Copy, Default)]
pub struct PixelAttrs {
    value: u8,
}

impl PixelAttrs {
    // # NOTE Bits 6 and 7 are used for priority

    const FIRST_TARGET: u8 = 0x1; // bit 0
    const SECOND_TARGET: u8 = 0x2; // bit 1
    const PALETTE_4BPP: u8 = 0x4; // bit 2
    const SEMI_TRANSPARENT: u8 = 0x8; // bit 3

    fn effects_mask(mut self, has_effects: bool) -> Self {
        if !has_effects {
            self.value &= !0xB; // mask out bits 0,1,3
        }
        self
    }

    pub fn is_first_target(&self) -> bool {
        (self.value & Self::FIRST_TARGET) != 0
    }

    pub fn is_second_target(&self) -> bool {
        (self.value & Self::SECOND_TARGET) != 0
    }

    pub fn set_first_target(&mut self) {
        self.value |= Self::FIRST_TARGET;
    }

    pub fn set_second_target(&mut self) {
        self.value |= Self::SECOND_TARGET;
    }

    /// Only used by OBJ layer pixels while calculating color.
    pub fn is_4bpp(&self) -> bool {
        (self.value & Self::PALETTE_4BPP) != 0
    }

    /// Only used by OBJ layer pixels
    pub fn set_4bpp(&mut self) {
        self.value |= Self::PALETTE_4BPP;
    }

    /// Only used by OBJ layer pixels
    pub fn set_8bpp(&mut self) {
        /* NOP */
    }

    pub fn set_semi_transparent(&mut self) {
        self.value |= Self::SEMI_TRANSPARENT;
    }

    pub fn is_semi_transparent(&self) -> bool {
        (self.value & Self::SEMI_TRANSPARENT) != 0
    }

    pub fn set_priority(&mut self, priority: u16) {
        self.value |= (priority as u8) << 6;
    }

    pub fn priority(&self) -> u16 {
        (self.value >> 6) as u16
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct LayerAttrs {
    value: u8,
}

impl LayerAttrs {
    const BITMAP_16BPP: u8 = 0x1;
    const PALETTE_4BPP: u8 = 0x2;

    pub fn is_bitmap(&self) -> bool {
        (self.value & Self::BITMAP_16BPP) != 0
    }

    pub fn is_4bpp(&self) -> bool {
        (self.value & Self::PALETTE_4BPP) != 0
    }

    pub fn set_bitmap(&mut self) {
        self.value |= Self::BITMAP_16BPP;
    }

    pub fn set_4bpp(&mut self) {
        self.value |= Self::PALETTE_4BPP;
    }

    pub fn set_8bpp(&mut self) {
        /* NOP */
    }
}

#[derive(Default, Copy, Clone)]
struct LineBits {
    inner: [u8; 30],
}

impl LineBits {
    const fn ones() -> Self {
        LineBits { inner: [0xFF; 30] }
    }

    const fn zeroes() -> Self {
        LineBits { inner: [0x00; 30] }
    }

    fn put(&mut self, index: usize, value: bool) {
        if index < 240 {
            self.inner[index / 8] |= (value as u8) << (index % 8);
        }
    }

    fn get(&self, index: usize) -> bool {
        if index < 240 {
            (self.inner[index / 8] & (1 << (index % 8))) != 0
        } else {
            false
        }
    }
}

#[derive(Copy, Clone)]
struct WindowMask {
    visible: LineBits,
    effects: LineBits,
}

impl WindowMask {
    fn new_all_enabled() -> Self {
        WindowMask {
            visible: LineBits::ones(),
            effects: LineBits::ones(),
        }
    }

    fn new_all_disabled() -> Self {
        WindowMask {
            visible: LineBits::zeroes(),
            effects: LineBits::zeroes(),
        }
    }

    fn set_visible(&mut self, x: usize, visible: bool, effects: bool) {
        if x < 240 {
            self.visible.put(x, visible);
            self.effects.put(x, effects);
        }
    }

    fn visible(&self, x: usize) -> bool {
        self.visible.get(x)
    }

    fn effects(&self, x: usize) -> bool {
        self.effects.get(x)
    }
}
