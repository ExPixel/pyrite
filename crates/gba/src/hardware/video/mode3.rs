use super::{
    line::{GbaLine, Layer, Pixel},
    RenderContext, VISIBLE_LINE_WIDTH,
};

pub(super) fn render(line: &mut GbaLine, _context: RenderContext) {
    for x in 0..VISIBLE_LINE_WIDTH {
        line.put(Layer::Bg2, x, Pixel::new(0xFFFF));
    }
}
