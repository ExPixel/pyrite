use util::bits::BitOps;

use crate::{hardware::palette::Palette, memory::VRAM_SIZE, video::registers::BgMode};

use super::{registers::GbaVideoRegisters, HBlankContext, VISIBLE_LINE_WIDTH};

pub struct GbaLine {
    pixels: [DoublePixel; VISIBLE_LINE_WIDTH],
}

impl GbaLine {
    pub fn push(&mut self, x: usize, pixel: Pixel) {
        self.pixels[x].push(pixel);
    }

    pub fn clear(&mut self, context: &Palette) {
        let pixel = Pixel::from(context.get_bg256(0));
        self.pixels.fill(DoublePixel::new(pixel, pixel));
    }

    pub fn blend(&mut self, output: &mut [u16; VISIBLE_LINE_WIDTH], context: BlendContext) {
        #[cfg(feature = "puffin")]
        puffin::profile_function!();

        let mode = context.registers.dispcnt.bg_mode();
        let is_bitmap_16bpp_mode = mode == BgMode::Mode3 || mode == BgMode::Mode5;

        if !is_bitmap_16bpp_mode {
            self.blend_internal::<false>(output, context);
        } else {
            self.blend_internal::<true>(output, context);
        }
    }

    fn blend_internal<const IS_BITMAP_16BPP_MODE: bool>(
        &self,
        output: &mut [u16; VISIBLE_LINE_WIDTH],
        context: BlendContext,
    ) {
        for (pixel, output) in self.pixels.iter().zip(output.iter_mut()) {
            if IS_BITMAP_16BPP_MODE {
                *output = pixel.top().color_16bpp();
            } else {
                let color = if pixel.top().attrs().is_obj() {
                    context.palette.get_obj256(pixel.top().entry())
                } else {
                    context.palette.get_bg256(pixel.top().entry())
                };
                *output = color;
            }
        }
    }
}

impl Default for GbaLine {
    fn default() -> Self {
        Self {
            pixels: [DoublePixel::default(); VISIBLE_LINE_WIDTH],
        }
    }
}

/// The top pixel is actually in the lower 16 bits of the u32, and the bottom
/// pixel is in the upper 16 bits. This saves us a shift when pushing the pixels.
#[derive(Default, Clone, Copy)]
struct DoublePixel(u32);

impl DoublePixel {
    pub fn new(top: Pixel, bottom: Pixel) -> Self {
        Self((u32::from(u16::from(top)) << 16) | u32::from(u16::from(bottom)))
    }

    pub fn top(&self) -> Pixel {
        Pixel::from(self.0 as u16)
    }

    pub fn bottom(&self) -> Pixel {
        Pixel::from((self.0 >> 16) as u16)
    }

    pub fn push(&mut self, pixel: Pixel) {
        let pixel = u32::from(u16::from(pixel));
        self.0 = (self.0 << 16) | pixel;
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

#[derive(Default, Clone, Copy)]
pub struct Pixel(u16);

impl Pixel {
    pub fn new(attrs: PixelAttrs, entry: u8) -> Self {
        Self(u16::from(entry) | u16::from(attrs))
    }

    pub fn new_bitmap(entry: u16) -> Self {
        Self(entry | u16::from(PixelAttrs::default().with_bitmap(true)))
    }

    pub fn entry(&self) -> u8 {
        self.0.get_bit_range(0..8) as u8
    }

    pub fn color_16bpp(&self) -> u16 {
        self.0
    }

    pub fn attrs(&self) -> PixelAttrs {
        PixelAttrs::from(self.0)
    }
}

impl From<u16> for Pixel {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<Pixel> for u16 {
    fn from(pixel: Pixel) -> Self {
        pixel.0
    }
}

#[derive(Clone, Copy, Default)]
pub struct PixelAttrs(u8);

impl PixelAttrs {
    const BITMAP_16BPP: u32 = 7;
    const OBJ: u32 = 0;

    pub fn is_bitmap(&self) -> bool {
        self.0.get_bit(Self::BITMAP_16BPP)
    }

    pub fn with_bitmap(&self, value: bool) -> Self {
        Self(self.0.put_bit(Self::BITMAP_16BPP, value))
    }

    pub fn is_obj(&self) -> bool {
        self.0.get_bit(Self::OBJ)
    }

    pub fn with_obj(&self, value: bool) -> Self {
        Self(self.0.put_bit(Self::OBJ, value))
    }
}

impl From<u16> for PixelAttrs {
    fn from(value: u16) -> Self {
        Self((value >> 8) as u8)
    }
}

impl From<PixelAttrs> for u16 {
    fn from(attrs: PixelAttrs) -> Self {
        (attrs.0 as u16) << 8
    }
}
