use util::bits::BitOps;

#[cfg(feature = "nightly")]
use core::intrinsics::unlikely;
#[cfg(not(feature = "nightly"))]
use std::convert::identity as unlikely;

use crate::{
    alu::{multiply, BinaryOp, ExtractOp2, Psr},
    clock::Cycles,
    cpu::Cpu,
    memory::Memory,
    transfer::{BlockDataTransfer, IndexingMode, SDTCalculateOffset, SingleDataTransfer},
    AccessType, CpsrFlag, CpuException, CpuMode,
};

/// Branch
///
/// B <offset>
pub fn arm_b(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    let offset = (instr & 0xFFFFFF).sign_extend(24).wrapping_shl(2);
    let pc = cpu.registers.read(15);
    let dest = pc.wrapping_add(offset);
    cpu.branch_arm(dest, memory)
}

/// Branch and Link
///
/// BL <offset>
pub fn arm_bl(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    let offset = (instr & 0xFFFFFF).sign_extend(24).wrapping_shl(2);
    let pc = cpu.registers.read(15);
    let dest = pc.wrapping_add(offset);
    cpu.registers.write(14, pc.wrapping_sub(4));
    cpu.branch_arm(dest, memory)
}

/// Branch and Exchange
///
/// BX Rn
pub fn arm_bx(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    let destination = cpu.registers.read(instr.get_bit_range(0..=3));

    if destination.get_bit(0) {
        cpu.registers.set_flag(CpsrFlag::T);
        cpu.branch_thumb(destination, memory)
    } else {
        cpu.branch_arm(destination, memory)
    }
}

/// Data Processing Instruction
///
/// MOV,MVN (single operand instructions.)  
/// `<opcode>{cond}{S} Rd,<Op2>`
///
/// CMP,CMN,TEQ,TST (instructions which do not produce a result.)  
/// `<opcode>{cond} Rn,<Op2>`
///
/// AND,EOR,SUB,RSB,ADD,ADC,SBC,RSC,ORR,BIC  
/// `<opcode>{cond}{S} Rd,Rn,<Op2>`
pub fn arm_dataproc<O, const S: bool, E>(
    instr: u32,
    cpu: &mut Cpu,
    memory: &mut dyn Memory,
) -> Cycles
where
    O: BinaryOp,
    E: ExtractOp2,
{
    let rd = instr.get_bit_range(12..=15);
    let rn = instr.get_bit_range(16..=19);

    let mut lhs = cpu.registers.read(rn);
    let mut cycles = E::stall();

    // When using R15 as operand (Rm or Rn), the returned value
    // depends on the instruction: PC+12 if I=0,R=1 (shift by register),
    // otherwise PC+8 (shift by immediate).
    if rn == 15 && E::IS_REGISTER_SHIFT {
        lhs = lhs.wrapping_add(4);
    }

    let rhs = E::extract::<S>(instr, &mut cpu.registers);
    let result = O::execute(&cpu.registers, lhs, rhs);
    O::set_flags_if::<S>(&mut cpu.registers, lhs, rhs, result);

    // If S=1, Rd=R15; should not be used in user mode:
    //   CPSR = SPSR_<current mode>
    //   PC = result
    //   For example: MOVS PC,R14  ;return from SWI (PC=R14_svc, CPSR=SPSR_svc).
    if unlikely(rd == 15 && S) {
        cpu.registers.write_cpsr(cpu.registers.read_spsr());
        cycles += cpu.branch(result, memory);
    } else if unlikely(rd == 15 && O::HAS_RESULT) {
        cycles += cpu.branch(result, memory);
    } else if O::HAS_RESULT {
        cpu.registers.write(rd, result);
    } else {
        O::set_flags_if::<S>(&mut cpu.registers, lhs, rhs, result);
    }

    cycles
}

/// Single Data Transfer (LDR, STR)
///
/// `<LDR|STR>{cond}{B}{T} Rd,<Address>`
pub fn arm_single_data_transfer<T, O, I, const WRITEBACK: bool>(
    instr: u32,
    cpu: &mut Cpu,
    memory: &mut dyn Memory,
) -> Cycles
where
    T: SingleDataTransfer,
    O: SDTCalculateOffset,
    I: IndexingMode,
{
    let rd = instr.get_bit_range(12..=15);
    let rn = instr.get_bit_range(16..=19);

    let offset = O::calculate_offset(instr, &mut cpu.registers);
    let mut address = cpu.registers.read(rn);
    address = I::calculate_single_data_transfer_address(address, offset);
    let mut cycles = T::transfer(rd, address, &mut cpu.registers, memory);

    if WRITEBACK {
        // FIXME At this point Rn is not allowed be r15 but I'm not sure if I should
        //       assert that and panic here or just log an error. No logging facilities
        //       for this part of the code at the moment though so once I figure that out
        //       I should probably take a look at this. For now I just branch anyway later.
        // From ARM Documentation:
        //      Write-back must not be specified if R15 is specified as the base register (Rn).
        //      When using R15 as the base register.
        address = I::calculate_single_data_transfer_writeback_address(address, offset);
        cpu.registers.write(rn, address);
    }

    // During the third cycle, the ARM7TDMI-S processor transfers the data to the
    // destination register. (External memory is not used.) Normally, the ARM7TDMI-S
    // core merges this third cycle with the next prefetch to form one memory N-cycle
    if T::IS_LOAD {
        cycles += Cycles::one();
    }

    if T::IS_LOAD && (rd == 15 || (WRITEBACK && rn == 15)) {
        let destination = cpu.registers.read(15);
        cycles += cpu.branch_arm(destination, memory);
    }

    if !T::IS_LOAD {
        cpu.next_fetch_access_type = AccessType::NonSequential;
    }

    cycles
}

/// Block Data Transfer (LDM, STM)
///
/// `<LDM|STM>{cond}<FD|ED|FA|EA|IA|IB|DA|DB> Rn{!},<Rlist>{^}`  
pub fn arm_block_data_transfer<T, I, const WRITEBACK: bool, const S: bool>(
    instr: u32,
    cpu: &mut Cpu,
    memory: &mut dyn Memory,
) -> Cycles
where
    T: BlockDataTransfer,
    I: IndexingMode,
{
    let register_list = instr.get_bit_range(0..=15);
    let rn = instr.get_bit_range(16..=19);
    let base_address = cpu.registers.read(rn);
    let register_count = register_list.count_ones();

    let mut address = I::block_transfer_lowest_address(base_address, register_count);
    address = address.wrapping_sub(4); // we start with an add every loop iteration

    // If the S-bit is set for an LDM instruction which doesn't include R15 in the transfer
    // list or an STM instruction, then the registers transferred are taken from the user
    // bank.
    let force_user_mode = S && (!T::IS_LOAD || !register_list.get_bit(15));
    let starting_mode = cpu.registers.read_mode();
    if force_user_mode {
        cpu.registers.write_mode(CpuMode::User);
    }

    let mut cycles = Cycles::zero();
    let mut access_type = AccessType::NonSequential;

    for register in 0..16 {
        if !register_list.get_bit(register) {
            continue;
        }
        address = address.wrapping_add(4);
        cycles += T::transfer(register, address, access_type, &mut cpu.registers, memory);

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
            if WRITEBACK && !T::IS_LOAD {
                let writeback_address =
                    I::calculate_block_transfer_writeback_address(base_address, register_count);
                cpu.registers.write(rn, writeback_address);
            }
        }
    }

    // if the S-bit is set in an LDM instruction and R15 is in the transfer list
    // then SPSR_<mode> is transferred to CPSR at the same time as R15 is loaded (the end
    // of the transfer).
    let load_spsr = S && T::IS_LOAD && register_list.get_bit(15);
    if load_spsr {
        cpu.registers.write_cpsr(cpu.registers.read_spsr());
    }

    if WRITEBACK && T::IS_LOAD {
        let writeback_address =
            I::calculate_block_transfer_writeback_address(base_address, register_count);
        cpu.registers.write(rn, writeback_address);
    }

    if force_user_mode {
        cpu.registers.write_mode(starting_mode);
    }

    if T::IS_LOAD && register_list.get_bit(15) {
        let destination = cpu.registers.read(15);
        if load_spsr && cpu.registers.get_flag(CpsrFlag::T) {
            cycles += cpu.branch_thumb(destination, memory);
        } else {
            cycles += cpu.branch_arm(destination, memory);
        }
    }

    if !T::IS_LOAD {
        cpu.next_fetch_access_type = AccessType::NonSequential;
    }

    cycles
}

/// Move value to status word
///
/// MSR - transfer register contents to PSR  
/// `MSR{cond} <psr>,Rm`
///
/// MSR - transfer register contents to PSR flag bits only  
/// `MSR{cond} <psrf>,Rm`  
/// The most significant four bits of the register contents are written to the N,Z,C
/// & V flags respectively.
///
/// MSR - transfer immediate value to PSR flag bits only  
/// `MSR{cond} <psrf>,<#expression>`
pub fn arm_msr<P, E>(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles
where
    P: Psr,
    E: ExtractOp2,
{
    let src = E::extract::<false>(instr, &mut cpu.registers);
    let flag_bits_only = (instr & 0x00010000) == 0;

    if flag_bits_only {
        P::write_flags_only(src, &mut cpu.registers);
    } else {
        P::write(src, &mut cpu.registers);
    }

    Cycles::zero()
}

/// Move status word to register
///
/// MRS{cond} Rd,<psr>
pub fn arm_mrs<P>(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles
where
    P: Psr,
{
    let src = P::read(&cpu.registers);
    let dst = instr.get_bit_range(12..=15);
    cpu.registers.write(dst, src);
    Cycles::zero()
}

/// Multiply and Multiply-Accumulate
///
/// MUL{cond}{S} Rd,Rm,Rs  
/// MLA{cond}{S} Rd,Rm,Rs,Rn
pub fn arm_mul<const S: bool, const A: bool>(
    instr: u32,
    cpu: &mut Cpu,
    _memory: &mut dyn Memory,
) -> Cycles {
    let rm = instr.get_bit_range(0..=3);
    let rs = instr.get_bit_range(8..=11);
    let rd = instr.get_bit_range(16..=19);

    let lhs = cpu.registers.read(rm);
    let rhs = cpu.registers.read(rs);
    let mut result = lhs.wrapping_mul(rhs);

    let acc_cycles = if A {
        let rn = instr.get_bit_range(12..=15);
        let accumulate = cpu.registers.read(rn);
        result = result.wrapping_add(accumulate);
        Cycles::one()
    } else {
        Cycles::zero()
    };

    if S {
        multiply::set_multiply_flags(result, &mut cpu.registers);
    }

    cpu.registers.write(rd, result);
    acc_cycles + multiply::internal_multiply_cycles(rhs)
}

/// Multiply Long and Multiply-Accumulate Long
///
/// UMULL{cond}{S} RdLo,RdHi,Rm,Rs  
/// UMLAL{cond}{S} RdLo,RdHi,Rm,Rs  
/// SMULL{cond}{S} RdLo,RdHi,Rm,Rs  
/// SMLAL{cond}{S} RdLo,RdHi,Rm,Rs  
pub fn arm_mul_long<const SIGNED: bool, const S: bool, const A: bool>(
    instr: u32,
    cpu: &mut Cpu,
    _memory: &mut dyn Memory,
) -> Cycles {
    let rm = instr.get_bit_range(0..=3);
    let rs = instr.get_bit_range(8..=11);
    let rd_lo = instr.get_bit_range(12..=15);
    let rd_hi = instr.get_bit_range(16..=19);

    let lhs = cpu.registers.read(rm) as u64;
    let rhs = cpu.registers.read(rs) as u64;

    let lhs = if SIGNED { lhs.sign_extend(32) } else { lhs };
    let rhs = if SIGNED { rhs.sign_extend(32) } else { rhs };

    let (acc, acc_cycles) = if A {
        let acc_lo = cpu.registers.read(rd_lo) as u64;
        let acc_hi = cpu.registers.read(rd_hi) as u64;
        ((acc_hi << 32) | acc_lo, Cycles::one())
    } else {
        (0, Cycles::zero())
    };

    let result = lhs.wrapping_mul(rhs).wrapping_add(acc);

    if S {
        multiply::set_multiply_flags(result, &mut cpu.registers);
    }

    cpu.registers.write(rd_lo, result as u32);
    cpu.registers.write(rd_hi, (result >> 32) as u32);
    acc_cycles + multiply::internal_multiply_cycles(rhs as u32)
}

/// Swap registers with memory word/byte
///
/// `<SWP>{cond}{B} Rd,Rm,[Rn]`  
pub fn arm_swp<const BYTE: bool>(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    let rn = instr.get_bit_range(16..=19);
    let rd = instr.get_bit_range(12..=15);
    let rm = instr.get_bit_range(0..=3);

    let address = cpu.registers.read(rn);
    let source = cpu.registers.read(rm);

    if BYTE {
        let (temp, wait_load) = memory.load8(address, AccessType::NonSequential);
        cpu.registers.write(rd, temp as u32);
        let wait_store = memory.store8(address, source as u8, AccessType::NonSequential);
        Cycles::one() + wait_load + wait_store
    } else {
        let (temp, wait_load) = memory.load32(address, AccessType::NonSequential);
        cpu.registers.write(rd, temp);
        let wait_store = memory.store32(address, source, AccessType::NonSequential);
        Cycles::one() + wait_load + wait_store
    }
}

/// Software Interrupt (SWI)
///
/// `SWI{cond} <expression>`  
pub fn arm_swi(_instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    cpu.exception_internal(CpuException::Swi, memory)
}

pub fn arm_undefined(_instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    cpu.exception_internal(CpuException::Undefined, memory)
}

/// ARM9
pub fn arm_blx(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    arm_undefined(instr, cpu, memory)
}

/// ARM9
pub fn arm_bkpt(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    arm_undefined(instr, cpu, memory)
}

/// ARM9
pub fn arm_clz(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    arm_undefined(instr, cpu, memory)
}

/// Used for unsupported M-Extension instructions
pub fn arm_m_extension_undefined(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    arm_undefined(instr, cpu, memory)
}

/// Unimplemented coprocessor functions.
pub fn arm_coprocessor_instr(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles {
    let address = cpu.registers.read(15).wrapping_sub(8);
    tracing::debug!(
        address = display(format_args!("0x{:08X}", address)),
        instruction = display(format_args!("0x{:08X}", instr)),
        "unimplemented ARM coprocessor instruction"
    );
    Cycles::one()
}
