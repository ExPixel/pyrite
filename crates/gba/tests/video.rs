use arm::disasm::MemoryView as _;
use common::{audio_noop, execute_until};
use gba::{
    video::{rgb5, LineBuffer, VISIBLE_LINE_COUNT, VISIBLE_LINE_WIDTH},
    Gba,
};

#[macro_use]
mod common;

#[test]
pub fn simple_mode3_test() {
    let mut frame_buffer = [[0u16; VISIBLE_LINE_WIDTH]; VISIBLE_LINE_COUNT];

    let done = |gba: &mut Gba| gba.mapped.view32(0x02000000) == 0xDEADBEEF;
    let video = |line: usize, data: &LineBuffer| frame_buffer[line].copy_from_slice(data);
    let audio = audio_noop;
    let _gba = execute_until("../../roms/custom/mode3-test.gba", done, video, audio);

    let center = VISIBLE_LINE_COUNT / 2;
    let line_width = 8;
    let line_y_min = center - (line_width / 2);
    let line_y_max = center + (line_width / 2);

    for (y, line) in frame_buffer.iter().enumerate() {
        let expected = if y >= line_y_min && y <= line_y_max {
            rgb5(24, 10, 24)
        } else {
            rgb5(16, 28, 16)
        };

        for &pixel in line.iter() {
            assert_eq!(
                expected, pixel,
                "incorrect pixel on line {y}: expected=0x{expected:04X}, found=0x{pixel:04X}"
            );
        }
    }
}
