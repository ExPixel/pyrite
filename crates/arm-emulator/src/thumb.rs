use util::bits::BitOps;

use crate::{
    alu::{self, BinaryOp, ExtractThumbOperand},
    cpu::{check_condition, Cpu},
    memory::Memory,
    transfer::{BlockDataTransfer, IndexingMode, SDTCalculateOffset, SingleDataTransfer},
    AccessType, CpsrFlag, CpuException, Cycles, Registers,
};

/// move shifted register
///
/// `LSL Rd, Rs, #Offset5`  
/// `LSR Rd, Rs, #Offset5`  
/// `ASR Rd, Rs, #Offset5`  
pub fn thumb_move_shifted_register<O>(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles
where
    O: BinaryOp,
{
    let rd = instr.get_bit_range(0..=2);
    let rs = instr.get_bit_range(3..=5);
    let lhs = cpu.registers.read(rs);
    let rhs = O::transform_imm_rhs(instr.get_bit_range(6..=10));
    let result = O::execute(&cpu.registers, lhs, rhs);
    O::set_flags(&mut cpu.registers, lhs, rhs, result);
    debug_assert!(O::HAS_RESULT);
    cpu.registers.write(rd, result);
    Cycles::zero()
}

/// move/compare/add/subtract immediate
///
/// `MOV Rd, #Offset8`  
/// `CMP Rd, #Offset8`  
/// `ADD Rd, #Offset8`  
/// `ADD Rd, #Offset8`  
pub fn thumb_mov_compare_add_subtract_imm<const RD: u32, O>(
    instr: u32,
    cpu: &mut Cpu,
    _memory: &mut dyn Memory,
) -> Cycles
where
    O: BinaryOp,
{
    let lhs = cpu.registers.read(RD);
    let rhs = O::transform_imm_rhs(instr.get_bit_range(0..=7));
    let result = O::execute(&cpu.registers, lhs, rhs);
    O::set_flags(&mut cpu.registers, lhs, rhs, result);
    if O::HAS_RESULT {
        cpu.registers.write(RD, result);
    }
    Cycles::zero()
}

/// add/subtract
///
/// `ADD Rd, Rs, Rn`  
/// `ADD Rd, Rs, #Offset3`  
/// `SUB Rd, Rs, Rn`  
/// `SUB Rd, Rs, #Offset3`  
pub fn thumb_add_subtract<E, O>(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles
where
    E: ExtractThumbOperand,
    O: BinaryOp,
{
    let rs = instr.get_bit_range(3..=5);
    let rd = instr.get_bit_range(0..=2);
    let lhs = cpu.registers.read(rs);
    let rhs = E::extract(instr, &cpu.registers);
    let result = O::execute(&cpu.registers, lhs, rhs);
    O::set_flags(&mut cpu.registers, lhs, rhs, result);
    debug_assert!(O::HAS_RESULT);
    cpu.registers.write(rd, result);
    Cycles::zero()
}

/// ALU operations
///
/// `AND Rd, Rs`  
/// `EOR Rd, Rs`  
/// `LSL Rd, Rs`  
/// `LSL Rd, Rs`  
/// `LSR Rd, Rs`  
/// `ASR Rd, Rs`  
/// `ADC Rd, Rs`  
/// `SBC Rd, Rs`  
/// `ROR Rd, Rs`  
/// `TST Rd, Rs`  
/// `NEG Rd, Rs`  
/// `CMP Rd, Rs`  
/// `CMN Rd, Rs`  
/// `ORR Rd, Rs`  
/// `MUL Rd, Rs`  
/// `BIC Rd, Rs`  
/// `MVN Rd, Rs`  
pub fn thumb_alu_operation(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles {
    let rd = instr.get_bit_range(0..=2);
    let rs = instr.get_bit_range(3..=5);
    let lhs = cpu.registers.read(rd);
    let rhs = cpu.registers.read(rs);

    let op = instr.get_bit_range(6..=9);
    let registers = &mut cpu.registers;
    match op {
        0x0 => thumb_alu_operation_internal::<alu::AndOp>(lhs, rhs, rd, registers),
        0x1 => thumb_alu_operation_internal::<alu::EorOp>(lhs, rhs, rd, registers),
        0x2 => thumb_alu_operation_internal::<alu::LslOp>(lhs, rhs, rd, registers),
        0x3 => thumb_alu_operation_internal::<alu::LsrOp>(lhs, rhs, rd, registers),
        0x4 => thumb_alu_operation_internal::<alu::AsrOp>(lhs, rhs, rd, registers),
        0x5 => thumb_alu_operation_internal::<alu::AdcOp>(lhs, rhs, rd, registers),
        0x6 => thumb_alu_operation_internal::<alu::SbcOp>(lhs, rhs, rd, registers),
        0x7 => thumb_alu_operation_internal::<alu::RorOp>(lhs, rhs, rd, registers),
        0x8 => thumb_alu_operation_internal::<alu::TstOp>(lhs, rhs, rd, registers),
        0x9 => thumb_alu_operation_internal::<alu::NegOp>(lhs, rhs, rd, registers),
        0xA => thumb_alu_operation_internal::<alu::CmpOp>(lhs, rhs, rd, registers),
        0xB => thumb_alu_operation_internal::<alu::CmnOp>(lhs, rhs, rd, registers),
        0xC => thumb_alu_operation_internal::<alu::OrrOp>(lhs, rhs, rd, registers),
        0xD => thumb_alu_operation_internal::<alu::MulOp>(lhs, rhs, rd, registers),
        0xE => thumb_alu_operation_internal::<alu::BicOp>(lhs, rhs, rd, registers),
        0xF => thumb_alu_operation_internal::<alu::MvnOp>(lhs, rhs, rd, registers),
        _ => unreachable!(),
    };

    if op != 0xD {
        Cycles::zero()
    } else {
        alu::multiply::internal_multiply_cycles(rhs)
    }
}

fn thumb_alu_operation_internal<O>(lhs: u32, rhs: u32, rd: u32, registers: &mut Registers)
where
    O: BinaryOp,
{
    let result = O::execute(registers, lhs, rhs);
    O::set_flags(registers, lhs, rhs, result);
    if O::HAS_RESULT {
        registers.write(rd, result)
    }
}

/// Hi register operations
///
/// `ADD Rd, Hs`  
/// `ADD Hd, Rs`  
/// `ADD Hd, Hs`  
/// `CMP Rd, Hs`  
/// `CMP Hd, Rs`  
/// `CMP Hd, Hs`  
/// `MOV Rd, Hs`  
/// `MOV Hd, Rs`  
/// `MOV Hd, Hs`  
pub fn thumb_hi_register_op<O>(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles
where
    O: BinaryOp,
{
    let rs_hi = instr.get_bit(6);
    let rd_hi = instr.get_bit(7);

    let rd = instr.get_bit_range(0..=2) + (if rd_hi { 8 } else { 0 });
    let rs = instr.get_bit_range(3..=5) + (if rs_hi { 8 } else { 0 });

    let lhs = cpu.registers.read(rd);
    let rhs = cpu.registers.read(rs);
    let result = O::execute(&cpu.registers, lhs, rhs);
    O::set_flags(&mut cpu.registers, lhs, rhs, result);
    if O::HAS_RESULT {
        cpu.registers.write(rd, result);
    }
    Cycles::zero()
}

/// branch exchange
///
/// `BX Rs`  
/// `BX Hs`  
pub fn thumb_bx(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    let rs_hi = instr.get_bit(6);
    let rs = instr.get_bit_range(3..=5) + (if rs_hi { 8 } else { 0 });
    let destination = cpu.registers.read(rs);

    if destination.get_bit(0) {
        cpu.branch_thumb(destination, memory)
    } else {
        cpu.registers.clear_flag(CpsrFlag::T);
        cpu.branch_arm(destination, memory)
    }
}

pub fn thumb_single_data_transfer<Transfer, DestinationRegister, BaseAddress, Offset, Indexing>(
    instr: u32,
    cpu: &mut Cpu,
    memory: &mut dyn Memory,
) -> Cycles
where
    Transfer: SingleDataTransfer,
    DestinationRegister: ExtractThumbOperand,
    BaseAddress: ExtractThumbOperand,
    Offset: SDTCalculateOffset,
    Indexing: IndexingMode,
{
    let rd = DestinationRegister::extract(instr, &cpu.registers);
    let offset = Offset::calculate_offset(instr, &mut cpu.registers);
    let mut address = BaseAddress::extract(instr, &cpu.registers);
    address = Indexing::calculate_single_data_transfer_address(address, offset);
    let mut cycles = Transfer::transfer(rd, address, &mut cpu.registers, memory);

    // During the third cycle, the ARM7TDMI-S processor transfers the data to the
    // destination register. (External memory is not used.) Normally, the ARM7TDMI-S
    // core merges this third cycle with the next prefetch to form one memory N-cycle
    if Transfer::IS_LOAD {
        cycles += Cycles::one();
    }

    if Transfer::IS_LOAD && rd == 15 {
        let destination = cpu.registers.read(15);
        cycles += cpu.branch_thumb(destination, memory);
    }

    if !Transfer::IS_LOAD {
        cpu.next_fetch_access_type = AccessType::NonSequential;
    }

    cycles
}

pub fn thumb_block_data_transfer<Transfer, BaseAddressRegister, Rlist, Indexing>(
    instr: u32,
    cpu: &mut Cpu,
    memory: &mut dyn Memory,
) -> Cycles
where
    Transfer: BlockDataTransfer,
    BaseAddressRegister: ExtractThumbOperand,
    Rlist: ExtractThumbOperand,
    Indexing: IndexingMode,
{
    let register_list = Rlist::extract(instr, &cpu.registers);
    let rn = BaseAddressRegister::extract(instr, &cpu.registers);
    let base_address = cpu.registers.read(rn);
    let register_count = register_list.count_ones();

    let mut address = Indexing::block_transfer_lowest_address(base_address, register_count);
    address = address.wrapping_sub(4); // we start with an add every loop iteration

    let mut cycles = Cycles::zero();
    let mut access_type = AccessType::NonSequential;

    for register in 0..16 {
        if !register_list.get_bit(register) {
            continue;
        }
        address = address.wrapping_add(4);
        cycles += Transfer::transfer(register, address, access_type, &mut cpu.registers, memory);

        if access_type == AccessType::NonSequential {
            access_type = AccessType::Sequential;

            // From ARM Documentation:
            //     When write-back is specified, the base is written back at the end of the second cycle
            //     of the instruction. During a STM, the first register is written out at the start of the
            //     second cycle. A STM which includes storing the base, with the base as the first register
            //     to be stored, will therefore store the unchanged value, whereas with the base second
            //     or later in the transfer order, will store the modified value. A LDM will always overwrite
            //     the updated base if the base is in the list.
            //
            // From Other ARM Documentation For LDM:
            //     During the third cycle, the first word is moved to the appropriate destination register
            //     while the second word is fetched from memory, and the modified base is latched
            //     internally in case it is needed to restore processor state after an abort.
            if !Transfer::IS_LOAD {
                let writeback_address = Indexing::calculate_block_transfer_writeback_address(
                    base_address,
                    register_count,
                );
                cpu.registers.write(rn, writeback_address);
            }
        }
    }

    if Transfer::IS_LOAD {
        let writeback_address =
            Indexing::calculate_block_transfer_writeback_address(base_address, register_count);
        cpu.registers.write(rn, writeback_address);
    }

    if Transfer::IS_LOAD && register_list.get_bit(15) {
        let destination = cpu.registers.read(15);
        cycles += cpu.branch_thumb(destination, memory);
    }

    if !Transfer::IS_LOAD {
        cpu.next_fetch_access_type = AccessType::NonSequential;
    }

    cycles
}

/// load address
///
/// `ADD Rd, PC, #Imm`  
/// `ADD Rd, SP, #Imm`  
pub fn thumb_load_address<const RD: u32, L>(
    instr: u32,
    cpu: &mut Cpu,
    _memory: &mut dyn Memory,
) -> Cycles
where
    L: ExtractThumbOperand,
{
    let lhs = L::extract(instr, &cpu.registers);
    let rhs = (instr & 0xFF) << 2;
    cpu.registers.write(RD, lhs.wrapping_add(rhs));
    Cycles::zero()
}

/// add offset to Stack Pointer
///
/// `ADD SP, #Imm`  
/// `ADD SP, #-Imm`  
pub fn thumb_add_sp(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles {
    let mut offset = instr.get_bit_range(0..=6) << 2;
    if instr.get_bit(7) {
        offset = -(offset as i32) as u32;
    }
    let sp = cpu.registers.read(13);
    cpu.registers.write(13, sp.wrapping_add(offset));
    Cycles::zero()
}

/// conditional branch
///
/// `B<COND> label`
pub fn thumb_conditional_branch<const CONDITION: u32>(
    instr: u32,
    cpu: &mut Cpu,
    memory: &mut dyn Memory,
) -> Cycles {
    if check_condition(CONDITION, &cpu.registers) {
        let offset = ((instr & 0xFF) << 1).sign_extend(9);
        let pc = cpu.registers.read(15);
        let dest = pc.wrapping_add(offset) & 0xFFFFFFFE;
        cpu.branch_thumb(dest, memory)
    } else {
        Cycles::zero()
    }
}

/// unconditional branch
///
/// `B label`
pub fn thumb_unconditional_branch(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    let offset = ((instr & 0x7FF) << 1).sign_extend(12);
    let pc = cpu.registers.read(15);
    let dest = pc.wrapping_add(offset) & 0xFFFFFFFE;
    cpu.branch_thumb(dest, memory)
}

// long branch with link (setup)
//
// `BL label`
pub fn thumb_bl_setup(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles {
    let pc = cpu.registers.read(15);
    let off = ((instr & 0x7FF) << 12).sign_extend(23);
    let setup = pc.wrapping_add(off);
    cpu.registers.write(14, setup);

    Cycles::zero()
}

// long branch with link (execute)
//
// `BL label`
pub fn thumb_bl_complete(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    let pc = cpu.registers.read(15);
    let lr = cpu.registers.read(14);
    let off = (instr & 0x7FF) << 1;
    let dest = lr.wrapping_add(off) & 0xFFFFFFFE;
    cpu.registers.write(14, (pc.wrapping_sub(2)) | 1);
    cpu.branch_thumb(dest, memory)
}

/// Software Interrupt (SWI)
///
/// `SWI{cond} <expression>`  
pub fn thumb_swi(_instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    cpu.exception_internal(CpuException::Swi, memory)
}

pub fn thumb_undefined(_instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    cpu.exception_internal(CpuException::Undefined, memory)
}

/// ARM9
pub fn thumb_blx(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    thumb_undefined(instr, cpu, memory)
}

/// ARM9
pub fn thumb_bkpt(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    thumb_undefined(instr, cpu, memory)
}
