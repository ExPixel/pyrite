use util::bits::BitOps;

use crate::{
    alu::{AriOp2, ExtractOp2, LliOp2, LriOp2, RriOp2},
    Cpu, CpuMode, Cycles, Memory, Registers,
};

pub struct Ldr<const USER_MODE: bool = false>;
pub struct Ldrb<const USER_MODE: bool = false>;
pub struct Str<const USER_MODE: bool = false>;
pub struct Strb<const USER_MODE: bool = false>;

pub struct Ldrh;
pub struct Strh;
pub struct Ldrsb;
pub struct Ldrsh;

pub struct PreIncrement;
pub struct PreDecrement;

pub struct PostIncrement;
pub struct PostDecrement;

pub struct HalfwordAndSignedImmOffset;
pub struct HalfwordAndSignedRegOffset;

pub struct Ldm;
pub struct Stm;

pub struct ThumbImm8ExtendedTo10;
pub struct ThumbImm5;
pub struct ThumbImm5ExtendedTo6;
pub struct ThumbImm5ExtendedTo7;
/// Common THUMB mode Ro (bits 6-8)
pub struct ThumbRegisterOffset;

impl<const USER_MODE: bool> SingleDataTransfer for Ldr<USER_MODE> {
    const IS_LOAD: bool = true;

    fn transfer(
        destination_register: u32,
        source_address: u32,
        cpu: &mut Cpu,
        memory: &mut dyn Memory,
    ) -> Cycles {
        let (mut value, wait) = if USER_MODE {
            // FIXME This doesn't really do anything on the GBA as far as I know
            //       But here for completeness I guess. Would make more sense if we
            //       passed the registers to memory whenever we made a read or
            //       write so that we would check things like the current address
            //       and mode.
            let old_mode = cpu.registers.write_mode(CpuMode::User);
            let (value, wait) = memory.load32(source_address & !0x3, cpu);
            cpu.registers.write_mode(old_mode);
            (value, wait)
        } else {
            memory.load32(source_address & !0x3, cpu)
        };

        // From the ARM7TDMI Documentation:
        //  A word load will normally use a word aligned address, however,
        //  an address offset from the word boundary will cause the data to
        //  be rotated into the register so that the addressed byte occupies bit 0-7.
        // Basically we rotate the word to the right by the number of bits that the address
        // is unaligned by (offset from the word boundary).
        value = value.rotate_right(8 * (source_address % 4));

        cpu.registers.write(destination_register, value);

        Cycles::one() + wait
    }
}

impl<const USER_MODE: bool> SingleDataTransfer for Ldrb<USER_MODE> {
    const IS_LOAD: bool = true;

    fn transfer(rd: u32, src_addr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
        let (value, wait) = memory.load8(src_addr, cpu);
        cpu.registers.write(rd, value as u32);
        Cycles::one() + wait
    }
}

impl<const USER_MODE: bool> SingleDataTransfer for Str<USER_MODE> {
    const IS_LOAD: bool = false;

    fn transfer(rd: u32, dst_addr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
        let mut value = cpu.registers.read(rd);

        // If the program counter is used as the source register in a word store, it will be
        // 12 bytes ahead instead of 8 when read.
        if rd == 15 {
            value = value.wrapping_add(4);
        }

        // FIXME    Not sure if this means that the behavior of an unaligned word store
        //          is completely handled by whatever is on the other end or if only
        //          work aligned addresses are used.
        //
        // From ARM documentation:
        //      A word store (STR) should generate a word aligned address. The word presented to
        //      the data bus is not affected if the address is not word aligned. That is, bit 31 of the
        //      register being stored always appears on data bus output 31.
        Cycles::one() + memory.store32(dst_addr & !0x3, value, cpu)
    }
}

impl<const USER_MODE: bool> SingleDataTransfer for Strb<USER_MODE> {
    const IS_LOAD: bool = false;

    fn transfer(rd: u32, dst_addr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
        let mut value = cpu.registers.read(rd);

        // If the program counter is used as the source register in a byte store, it will be
        // 12 bytes ahead instead of 8 when read.
        if rd == 15 {
            value = value.wrapping_add(4);
        }

        Cycles::one() + memory.store8(dst_addr, value as u8, cpu)
    }
}

impl SingleDataTransfer for Ldrh {
    const IS_LOAD: bool = true;

    fn transfer(rd: u32, addr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
        // We don't align the address here. If bit 0 is high then behavior is just
        // unpredictable (depends on memory hardware).
        let (value, wait) = memory.load16(addr, cpu);
        cpu.registers.write(rd, value as u32);
        Cycles::one() + wait
    }
}

impl SingleDataTransfer for Strh {
    const IS_LOAD: bool = false;

    fn transfer(rd: u32, addr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
        let mut value = cpu.registers.read(rd);

        // If the program counter is used as the source register in a halfword store, it will
        // be 12 bytes ahead instead of 8 when read.
        if rd == 15 {
            value = value.wrapping_add(4);
        }

        Cycles::one() + memory.store16(addr, value as u16, cpu)
    }
}

impl SingleDataTransfer for Ldrsb {
    const IS_LOAD: bool = true;

    fn transfer(rd: u32, addr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
        let (value, wait) = memory.load8(addr, cpu);
        cpu.registers.write(rd, value as i8 as i32 as u32);
        Cycles::one() + wait
    }
}

impl SingleDataTransfer for Ldrsh {
    const IS_LOAD: bool = true;

    fn transfer(rd: u32, addr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
        // We don't align the address here. If bit 0 is high then behavior is just
        // unpredictable (depends on memory hardware).
        let (value, wait) = memory.load16(addr, cpu);
        cpu.registers.write(rd, value as i16 as i32 as u32);
        Cycles::one() + wait
    }
}

pub struct SDTImmOffset;

impl SDTCalculateOffset for SDTImmOffset {
    #[inline(always)]
    fn calculate_offset(instr: u32, _registers: &mut Registers) -> u32 {
        instr & 0xFFF
    }
}

impl SDTCalculateOffset for AriOp2 {
    #[inline(always)]
    fn calculate_offset(instr: u32, registers: &mut Registers) -> u32 {
        <Self as ExtractOp2>::extract::<false>(instr, registers)
    }
}

impl SDTCalculateOffset for LliOp2 {
    #[inline(always)]
    fn calculate_offset(instr: u32, registers: &mut Registers) -> u32 {
        <Self as ExtractOp2>::extract::<false>(instr, registers)
    }
}

impl SDTCalculateOffset for LriOp2 {
    #[inline(always)]
    fn calculate_offset(instr: u32, registers: &mut Registers) -> u32 {
        <Self as ExtractOp2>::extract::<false>(instr, registers)
    }
}

impl SDTCalculateOffset for RriOp2 {
    #[inline(always)]
    fn calculate_offset(instr: u32, registers: &mut Registers) -> u32 {
        <Self as ExtractOp2>::extract::<false>(instr, registers)
    }
}

impl SDTCalculateOffset for HalfwordAndSignedImmOffset {
    #[inline(always)]
    fn calculate_offset(instr: u32, _registers: &mut Registers) -> u32 {
        let lo = instr.get_bit_range(0..=3);
        let hi = instr.get_bit_range(8..=11);
        lo | (hi << 4)
    }
}

impl SDTCalculateOffset for HalfwordAndSignedRegOffset {
    #[inline(always)]
    fn calculate_offset(instr: u32, registers: &mut Registers) -> u32 {
        let rm = instr.get_bit_range(0..=3);
        registers.read(rm)
    }
}

impl SDTCalculateOffset for ThumbImm8ExtendedTo10 {
    fn calculate_offset(instr: u32, _registers: &mut Registers) -> u32 {
        (instr & 0xFF) << 2
    }
}

impl SDTCalculateOffset for ThumbImm5 {
    fn calculate_offset(instr: u32, _registers: &mut Registers) -> u32 {
        instr.get_bit_range(6..=10)
    }
}

impl SDTCalculateOffset for ThumbImm5ExtendedTo6 {
    fn calculate_offset(instr: u32, _registers: &mut Registers) -> u32 {
        instr.get_bit_range(6..=10) << 1
    }
}

impl SDTCalculateOffset for ThumbImm5ExtendedTo7 {
    fn calculate_offset(instr: u32, _registers: &mut Registers) -> u32 {
        instr.get_bit_range(6..=10) << 2
    }
}

impl SDTCalculateOffset for ThumbRegisterOffset {
    fn calculate_offset(instr: u32, registers: &mut Registers) -> u32 {
        let ro = instr.get_bit_range(6..=8);
        registers.read(ro)
    }
}

impl IndexingMode for PreIncrement {
    #[inline(always)]
    fn calculate_single_data_transfer_address(address: u32, offset: u32) -> u32 {
        address.wrapping_add(offset)
    }

    #[inline(always)]
    fn block_transfer_lowest_address(base_address: u32, _register_count: u32) -> u32 {
        base_address.wrapping_add(4)
    }

    fn calculate_block_transfer_writeback_address(base_address: u32, register_count: u32) -> u32 {
        base_address.wrapping_add(register_count * 4)
    }
}

impl IndexingMode for PreDecrement {
    #[inline(always)]
    fn calculate_single_data_transfer_address(address: u32, offset: u32) -> u32 {
        address.wrapping_sub(offset)
    }

    #[inline(always)]
    fn block_transfer_lowest_address(base_address: u32, register_count: u32) -> u32 {
        base_address.wrapping_sub(register_count * 4)
    }

    fn calculate_block_transfer_writeback_address(base_address: u32, register_count: u32) -> u32 {
        base_address.wrapping_sub(register_count * 4)
    }
}

impl IndexingMode for PostIncrement {
    #[inline(always)]
    fn calculate_single_data_transfer_writeback_address(address: u32, offset: u32) -> u32 {
        address.wrapping_add(offset)
    }

    #[inline(always)]
    fn block_transfer_lowest_address(base_address: u32, _register_count: u32) -> u32 {
        base_address
    }

    fn calculate_block_transfer_writeback_address(base_address: u32, register_count: u32) -> u32 {
        base_address.wrapping_add(register_count * 4)
    }
}

impl IndexingMode for PostDecrement {
    #[inline(always)]
    fn calculate_single_data_transfer_writeback_address(address: u32, offset: u32) -> u32 {
        address.wrapping_sub(offset)
    }

    #[inline(always)]
    fn block_transfer_lowest_address(base_address: u32, register_count: u32) -> u32 {
        base_address
            .wrapping_sub(register_count * 4)
            .wrapping_add(4)
    }

    fn calculate_block_transfer_writeback_address(base_address: u32, register_count: u32) -> u32 {
        base_address.wrapping_sub(register_count * 4)
    }
}

pub trait SDTCalculateOffset {
    fn calculate_offset(instr: u32, registers: &mut Registers) -> u32;
}

pub trait IndexingMode {
    /// Address that is used for the transfer (pre-index)
    #[inline(always)]
    fn calculate_single_data_transfer_address(address: u32, _offset: u32) -> u32 {
        address
    }

    /// Address that is used for writeback (post-index)
    #[inline(always)]
    fn calculate_single_data_transfer_writeback_address(address: u32, _offset: u32) -> u32 {
        address
    }

    /// Calculates the lowest address of a block data transfer.
    fn block_transfer_lowest_address(base_address: u32, register_count: u32) -> u32;

    fn calculate_block_transfer_writeback_address(base_address: u32, register_count: u32) -> u32;
}

impl BlockDataTransfer for Ldm {
    const IS_LOAD: bool = true;

    #[inline]
    fn transfer(
        destination_register: u32,
        source_address: u32,
        cpu: &mut Cpu,
        memory: &mut dyn Memory,
    ) -> Cycles {
        let (value, wait) = memory.load32(source_address, cpu);
        cpu.registers.write(destination_register, value);
        Cycles::one() + wait
    }
}

impl BlockDataTransfer for Stm {
    const IS_LOAD: bool = false;

    #[inline]
    fn transfer(
        source_register: u32,
        destination_address: u32,
        cpu: &mut Cpu,
        memory: &mut dyn Memory,
    ) -> Cycles {
        let mut value = cpu.registers.read(source_register);
        // When r15 is stored as part of an STM instruction it will 12 bytes ahead instead of 8.
        // NOTE: Thumb mode cannot store r15 (only load), so we ignore it here and only handle ARM.
        if source_register == 15 {
            value = value.wrapping_add(4);
        }
        let wait = memory.store32(destination_address, value, cpu);
        Cycles::one() + wait
    }
}

pub trait SingleDataTransfer {
    const IS_LOAD: bool;

    fn transfer(rd: u32, addr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles;
}

pub trait BlockDataTransfer {
    const IS_LOAD: bool;

    fn transfer(register: u32, address: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles;
}
