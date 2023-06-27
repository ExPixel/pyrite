use crate::{
    alu::{AriOp2, ExtractOp2, LliOp2, LriOp2, RriOp2},
    CpuMode, Cycles, Memory, Registers,
};

pub struct Ldr<const USER_MODE: bool = false>;
pub struct LdrB<const USER_MODE: bool = false>;
pub struct Str<const USER_MODE: bool = false>;
pub struct StrB<const USER_MODE: bool = false>;

pub struct PreIncrement;
pub struct PreDecrement;

pub struct PostIncrement;
pub struct PostDecrement;

impl<const USER_MODE: bool> SingleDataTransfer for Ldr<USER_MODE> {
    const IS_LOAD: bool = true;

    fn transfer(
        rd: u32,
        src_addr: u32,
        registers: &mut Registers,
        memory: &mut dyn Memory,
    ) -> Cycles {
        let mut cycles = Cycles::zero();

        let mut value = if USER_MODE {
            // FIXME This doesn't really do anything on the GBA as far as I know
            //       But here for completeness I guess. Would make more sense if we
            //       passed the registers to memory whenever we made a read or
            //       write so that we would check things like the current address
            //       and mode.
            let old_mode = registers.write_mode(CpuMode::User);
            let v = memory.load32(src_addr & !0x3, Some(&mut cycles));
            registers.write_mode(old_mode);
            v
        } else {
            memory.load32(src_addr & !0x3, Some(&mut cycles))
        };

        // From the ARM7TDMI Documentation:
        //  A word load will normally use a word aligned address, however,
        //  an address offset from the word boundary will cause the data to
        //  be rotated into the register so that the addressed byte occupies bit 0-7.
        // Basically we rotate the word to the right by the number of bits that the address
        // is unaligned by (offset from the word boundary).
        value = value.rotate_right(8 * (src_addr % 4));

        registers.write(rd, value);
        cycles
    }
}

impl<const USER_MODE: bool> SingleDataTransfer for LdrB<USER_MODE> {
    const IS_LOAD: bool = true;

    fn transfer(
        rd: u32,
        src_addr: u32,
        registers: &mut Registers,
        memory: &mut dyn Memory,
    ) -> Cycles {
        let mut cycles = Cycles::zero();
        let value = memory.load8(src_addr, Some(&mut cycles));
        registers.write(rd, value as u32);
        cycles
    }
}

impl<const USER_MODE: bool> SingleDataTransfer for Str<USER_MODE> {
    const IS_LOAD: bool = false;

    fn transfer(
        rd: u32,
        dst_addr: u32,
        registers: &mut Registers,
        memory: &mut dyn Memory,
    ) -> Cycles {
        let mut value = registers.read(rd);

        // If the program counter is used as the source register in a word store, it will be
        // 12 bytes ahead instead of 8 when read.
        if rd == 15 {
            value = value.wrapping_add(4);
        }

        let mut cycles = Cycles::zero();
        memory.store32(dst_addr & !0x3, value, Some(&mut cycles));
        cycles
    }
}

impl<const USER_MODE: bool> SingleDataTransfer for StrB<USER_MODE> {
    const IS_LOAD: bool = false;

    fn transfer(
        rd: u32,
        dst_addr: u32,
        registers: &mut Registers,
        memory: &mut dyn Memory,
    ) -> Cycles {
        let mut value = registers.read(rd);

        // If the program counter is used as the source register in a byte store, it will be
        // 12 bytes ahead instead of 8 when read.
        if rd == 15 {
            value = value.wrapping_add(4);
        }

        let mut cycles = Cycles::zero();
        memory.store8(dst_addr, value as u8, Some(&mut cycles));
        cycles
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

impl SDTIndexingMode for PreIncrement {
    #[inline(always)]
    fn calculate_transfer_address(address: u32, offset: u32) -> u32 {
        address.wrapping_add(offset)
    }
}

impl SDTIndexingMode for PreDecrement {
    #[inline(always)]
    fn calculate_transfer_address(address: u32, offset: u32) -> u32 {
        address.wrapping_sub(offset)
    }
}

impl SDTIndexingMode for PostIncrement {
    #[inline(always)]
    fn calculate_writeback_address(address: u32, offset: u32) -> u32 {
        address.wrapping_add(offset)
    }
}

impl SDTIndexingMode for PostDecrement {
    #[inline(always)]
    fn calculate_writeback_address(address: u32, offset: u32) -> u32 {
        address.wrapping_sub(offset)
    }
}

pub trait SDTCalculateOffset {
    fn calculate_offset(instr: u32, registers: &mut Registers) -> u32;
}

pub trait SDTIndexingMode {
    /// Address that is used for the transfer (pre-index)
    #[inline(always)]
    fn calculate_transfer_address(address: u32, _offset: u32) -> u32 {
        address
    }

    /// Address that is used for writeback (post-index)
    #[inline(always)]
    fn calculate_writeback_address(address: u32, _offset: u32) -> u32 {
        address
    }
}

pub trait SingleDataTransfer {
    const IS_LOAD: bool;

    fn transfer(rd: u32, addr: u32, registers: &mut Registers, memory: &mut dyn Memory) -> Cycles;
}