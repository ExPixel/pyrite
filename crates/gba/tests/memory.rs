#[macro_use]
mod common;

#[test]
fn test_simple_gba_emulation_run() {
    let gba = emu_arm! {"
        ldr r0, =#0xDEADBEEF
        swi #0xCE
    "};
    assert_eq!(gba.cpu.registers.read(0), 0xDEADBEEF);
}
