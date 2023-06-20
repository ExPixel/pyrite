pub mod common;

#[test]
pub fn test_b() {
    let source = "\
b       _exit
mov     r0, #5";
    let (cpu, _mem) = common::execute_arm("b", source);
    assert_eq!(cpu.registers.read(0), 0);
}

#[test]
pub fn test_bl() {
    let source = "\
b       _exit
mov     r0, #5";
    let (cpu, _mem) = common::execute_arm("b", source);
    assert_eq!(cpu.registers.read(0), 0);
}
