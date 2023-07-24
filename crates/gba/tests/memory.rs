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

#[test]
fn test_32bit_read_from_bios() {
    // Reading from BIOS Memory (00000000-00003FFF)
    //      The BIOS memory is protected against reading, the GBA allows to read opcodes or data only if
    //      the program counter is located inside of the BIOS area. If the program counter is not in the
    //      BIOS area, reading will return the most recent successfully fetched BIOS opcode
    //
    // For our custom BIOS this would be
    //      E3A00000    mov r0, #0  @ <-- decoded
    //      E12FFF10    bx r0       @ <-- fetched
    let gba = emu_arm! {"
        ldr r1, =#0x0
        ldr r0, [r1]
        swi #0xCE
    "};
    assert_eq!(gba.cpu.registers.read(0), 0xE12FFF10);
}

#[test]
fn test_32bit_read_from_unused_memory() {
    // FIXME For now I only (crudely) emulate reading from unused memory. Will need something a bit more involved
    //       to emulate the other behaviors, but will do that a later time. -- Marc

    // Reading from Unused Memory (00004000-01FFFFFF,10000000-FFFFFFFF)
    //      Accessing unused memory at 00004000h-01FFFFFFh, and 10000000h-FFFFFFFFh (and 02000000h-03FFFFFFh
    //      when RAM is disabled via Port 4000800h) returns the recently pre-fetched opcode. For ARM code this is simply:
    //        WORD = [$+8]
    //      For THUMB code the result consists of two 16bit fragments and depends on the address area and
    //      alignment where the opcode was stored.
    //
    //      For THUMB code in Main RAM, Palette Memory, VRAM, and Cartridge ROM this is:
    //        LSW = [$+4], MSW = [$+4]
    //      For THUMB code in BIOS or OAM (and in 32K-WRAM on Original-NDS (in GBA mode)):
    //        LSW = [$+4], MSW = [$+6]   ;for opcodes at 4-byte aligned locations
    //        LSW = [$+2], MSW = [$+4]   ;for opcodes at non-4-byte aligned locations
    //      For THUMB code in 32K-WRAM on GBA, GBA SP, GBA Micro, NDS-Lite (but not NDS):
    //        LSW = [$+4], MSW = OldHI   ;for opcodes at 4-byte aligned locations
    //        LSW = OldLO, MSW = [$+4]   ;for opcodes at non-4-byte aligned locations
    //      Whereas OldLO/OldHI are usually:
    //        OldLO=[$+2], OldHI=[$+2]
    //      Unless the previous opcode's prefetch was overwritten; that can happen if the previous opcode was itself an LDR opcode,
    //      ie. if it was itself reading data:
    //        OldLO=LSW(data), OldHI=MSW(data)
    //        Theoretically, this might also change if a DMA transfer occurs.
    //
    //      Note: Additionally, as usually, the 32bit data value will be rotated if the data address wasn't 4-byte aligned,
    //      and the upper bits of the 32bit value will be masked in case of LDRB/LDRH reads.
    //
    //      Note: The opcode prefetch is caused by the prefetch pipeline in the CPU itself, not by the external gamepak prefetch,
    //      ie. it works for code in ROM and RAM as well.
    let gba = emu_arm! {"
        ldr r1, =#0x10000000
        ldr r0, [r1]
        swi #0xCE               @ <-- decoded
        mov r0, #0              @ <-- fetched
        bx  r0
    "};
    assert_eq!(gba.cpu.registers.read(0), 0xE3A00000);
}

#[test]
fn test_ewram_mirror_32bit() {
    let gba = emu_arm! {"
        ldr r1, =#0x0200000C
        ldr r2, =#0xDEADBEEF
        str r2, [r1]
        ldr r1, =#0x0204000C
        ldr r0, [r1]
        ldr r1, =#0x0208000C
        ldr r3, [r1]
        swi #0xCE
    "};

    assert_eq!(gba.cpu.registers.read(0), 0xDEADBEEF);
    assert_eq!(gba.cpu.registers.read(3), 0xDEADBEEF);
}

#[test]
fn test_ewram_mirror_16bit() {
    let gba = emu_arm! {"
        ldr r1, =#0x0200000C
        ldr r2, =#0xDEADBEEF
        strh r2, [r1]
        ldr r1, =#0x0204000C
        ldrh r0, [r1]
        ldr r1, =#0x0208000C
        ldrh r3, [r1]
        swi #0xCE
    "};

    assert_eq!(gba.cpu.registers.read(0), 0xBEEF);
    assert_eq!(gba.cpu.registers.read(3), 0xBEEF);
}

#[test]
fn test_ewram_mirror_8bit() {
    let gba = emu_arm! {"
        ldr r1, =#0x0200000C
        ldr r2, =#0xDEADBEEF
        strb r2, [r1]
        ldr r1, =#0x0204000C
        ldrb r0, [r1]
        ldr r1, =#0x0208000C
        ldrb r3, [r1]
        swi #0xCE
    "};

    assert_eq!(gba.cpu.registers.read(0), 0xEF);
    assert_eq!(gba.cpu.registers.read(3), 0xEF);
}
