import json
import os


def arm_instr_data_to_lut_entry(data):
    name = data["name"].lower()
    desc = data["desc"].lower() if "desc" in data else ""
    subname = data["subname"].lower() if "subname" in data else ""
    subdesc = data["subdesc"].lower() if "subdesc" in data else ""
    _class = data["_class"].lower() if "_class" in data else ""

    if name == "b":
        assert len(subname) == 0
        return "arm::arm_b"
    elif name == "bl":
        assert len(subname) == 0
        return "arm::arm_bl"
    elif name in DATAPROC_INSTR or name in DATAPROC_INSTR_S:
        if name.endswith("s"):
            s_flag = "S_FLAG_SET"
            op_name = name[0:-1].capitalize() + "Op"
        else:
            s_flag = "S_FLAG_CLR"
            op_name = name.capitalize() + "Op"

        if subname == "imm":
            op2 = "ImmOp2"
        elif subname == "lli":
            op2 = "LliOp2"
        elif subname == "llr":
            op2 = "LlrOp2"
        elif subname == "lri":
            op2 = "LriOp2"
        elif subname == "lrr":
            op2 = "LrrOp2"
        elif subname == "ari":
            op2 = "AriOp2"
        elif subname == "arr":
            op2 = "ArrOp2"
        elif subname == "rri":
            op2 = "RriOp2"
        elif subname == "rrr":
            op2 = "RrrOp2"
        else:
            op2 = None
            print("unknown dataproc subname: " + subname)

        return f"arm::arm_dataproc::<alu::{op_name}, {s_flag}, alu::{op2}>"

    elif name in [
        "ldr",
        "str",
        "ldrb",
        "strb",
        "ldrt",
        "strt",
        "ldrbt",
        "strbt",
        "ldrh",
        "strh",
        "ldrsb",
        "ldrsh",
    ]:
        op_name = name.capitalize()

        halfword_signed = name in ["ldrh", "strh", "ldrsb", "ldrsh"]
        tindex = op_name.rfind("t")
        if tindex > 2:
            op_name = op_name[0:tindex] + op_name[(tindex + 1) :]
            op_name = f"{op_name}<FORCE_USER_MODE>"

        writeback = "NO_WRITEBACK"

        if "post-increment" in subdesc:
            indexing = "PostIncrement"
            writeback = "WRITEBACK"

        elif "pre-increment" in subdesc:
            indexing = "PreIncrement"
            writeback = "WRITEBACK"
        elif "positive" in subdesc:
            indexing = "PreIncrement"

        elif "post-decrement" in subdesc:
            indexing = "PostDecrement"
            writeback = "WRITEBACK"

        elif "pre-decrement" in subdesc:
            indexing = "PreDecrement"
            writeback = "WRITEBACK"
        elif "negative" in subdesc:
            indexing = "PreDecrement"
        else:
            indexing = ""
            print("indexing not found: " + subname)
            return "ERROR"

        if not halfword_signed:
            if "immediate" in subdesc:
                offset = "SDTImmOffset"
            elif "arithmetic-right-shifted" in subdesc:
                offset = "alu::AriOp2"
            elif "right-shifted" in subdesc:
                offset = "alu::LriOp2"
            elif "left-shifted" in subdesc:
                offset = "alu::LliOp2"
            elif "right-rotated" in subdesc:
                offset = "alu::RriOp2"
            else:
                offset = ""
                print("offset not found: " + subname)
                return "ERROR"
        else:
            if "immediate offset" in subdesc:
                offset = "HalfwordAndSignedImmOffset"
            elif "register offset" in subdesc:
                offset = "HalfwordAndSignedRegOffset"
            else:
                offset = ""
                print("offset not found (halfword): " + subname)
                return "ERROR"
            pass

        return f"arm::arm_single_data_transfer::<{op_name}, {offset}, {indexing}, {writeback}>"

    elif name.startswith("ldm") or name.startswith("stm"):
        op_name = "Ldm" if name.startswith("l") else "Stm"

        if "u" in subname:
            s_flag = "S_FLAG_SET"
        else:
            s_flag = "S_FLAG_CLR"

        if "w" in subname:
            writeback = "WRITEBACK"
        else:
            writeback = "NO_WRITEBACK"

        if name.endswith("da"):
            indexing = "PostDecrement"
        elif name.endswith("ia"):
            indexing = "PostIncrement"
        elif name.endswith("db"):
            indexing = "PreDecrement"
        elif name.endswith("ib"):
            indexing = "PreIncrement"
        else:
            indexing = ""
            print("indexing not found: " + subname)
            return "ERROR"

        return f"arm::arm_block_data_transfer::<{op_name}, {indexing}, {writeback}, {s_flag}>"

    elif name == "mrs":
        if subname == "rc":
            return "arm::arm_mrs::<alu::Cpsr>"
        elif subname == "rs":
            return "arm::arm_mrs::<alu::Spsr>"
        else:
            print(f"unknown mrs subname: " + subname)
            return "ERROR"
    elif name == "msr":
        if subname == "rc":
            return "arm::arm_msr::<alu::Cpsr, alu::LliOp2>"
        elif subname == "rs":
            return "arm::arm_msr::<alu::Spsr, alu::LliOp2>"
        elif subname == "ic":
            return "arm::arm_msr::<alu::Cpsr, alu::ImmOp2>"
        elif subname == "is":
            return "arm::arm_msr::<alu::Spsr, alu::ImmOp2>"
        else:
            print(f"unknown msr subname: " + subname)
            return "ERROR"

    elif name in ["mul", "muls", "mla", "mlas"]:
        s_flag = "S_FLAG_SET" if name.endswith("s") else "S_FLAG_CLR"
        a_flag = "A_FLAG_SET" if name.startswith("mla") else "A_FLAG_CLR"
        return f"arm::arm_mul::<{s_flag}, {a_flag}>"

    elif name in [
        "umull",
        "umulls",
        "umlal",
        "umlals",
        "smull",
        "smulls",
        "smlal",
        "smlals",
    ]:
        signed = "SIGNED" if name.startswith("s") else "UNSIGNED"
        s_flag = "S_FLAG_SET" if name.endswith("s") else "S_FLAG_CLR"
        a_flag = "A_FLAG_SET" if "mla" in name else "A_FLAG_CLR"
        return f"arm::arm_mul_long::<{signed}, {s_flag}, {a_flag}>"

    elif name == "swi":
        return "arm::arm_swi"
    elif name == "bx":
        return "arm::arm_bx"
    elif name == "swp":
        return "arm::arm_swp::<SWP_WORD>"
    elif name == "swpb":
        return "arm::arm_swp::<SWP_BYTE>"

    elif name in ["stc", "ldc", "cdp", "mcr", "mrc"]:
        return "arm::arm_coprocessor_instr"

    elif name == "blx":
        return "arm::arm_blx"
    elif name == "bkpt":
        return "arm::arm_bkpt"
    elif name == "clz":
        return "arm::arm_clz"

    elif _class == "und":
        return "arm::arm_undefined"
    elif _class == "edsp":
        return "arm::arm_m_extension_undefined"

    print(f"unknown ARM instruction {name}/{subname} -- {desc} -- {subdesc}")
    return "arm::todo"


def thumb_instr_data_to_lut_entry(data):
    name = data["name"].lower()
    desc = data["desc"].lower() if "desc" in data else ""
    subname = data["subname"].lower() if "subname" in data else ""
    subdesc = data["subdesc"].lower() if "subdesc" in data else ""
    _class = data["_class"].lower() if "_class" in data else ""

    if name in ["lsl", "lsr", "asr"] and subname in "imm":
        op_name = name.capitalize() + "Op"
        return f"thumb::thumb_move_shifted_register::<alu::{op_name}>"
    elif name in ["mov", "cmp", "add", "sub"] and subname.startswith("i8r"):
        op_name = name.capitalize() + "Op"
        register = int(subname[3:])
        return (
            f"thumb::thumb_mov_compare_add_subtract_imm::<{register}, alu::{op_name}>"
        )
    elif name in ["add", "sub"] and (subname == "reg" or subname == "imm3"):
        op_name = name.capitalize() + "Op"
        imm = "AddSubtractImm3" if subname == "imm3" else "AddSubtractReg3"
        return f"thumb::thumb_add_subtract::<alu::{imm}, alu::{op_name}>"
    elif name == "dp":
        return f"thumb::thumb_alu_operation"
    elif name in ["addh", "cmph", "movh"]:
        op_name = name[:-1].capitalize() + "Op"
        return f"thumb::thumb_hi_register_op::<alu::{op_name}>"
    elif name == "bx":
        return f"thumb::thumb_bx"
    elif name == "ldrpc":
        register = int(subname[1:])
        return f"thumb::thumb_single_data_transfer::<Ldr, ConstReg<{register}>, WordAlignedPc, ThumbImm8ExtendedTo10, PreIncrement>"
    elif (
        name in ["str", "ldr", "strh", "strb", "ldrsb", "ldrh", "ldrb", "ldrsh"]
        and subname == "reg"
    ):
        op_name = name.capitalize()
        return f"thumb::thumb_single_data_transfer::<{op_name}, RegAt<0, 2>, RegAtValue<3, 5>, ThumbRegisterOffset, PreIncrement>"
    elif name in ["strb", "ldrb"] and subname == "imm5":
        op_name = name.capitalize()
        return f"thumb::thumb_single_data_transfer::<{op_name}, RegAt<0, 2>, RegAtValue<3, 5>, ThumbImm5, PreIncrement>"
    elif name in ["strh", "ldrh"] and subname == "imm5":
        op_name = name.capitalize()
        return f"thumb::thumb_single_data_transfer::<{op_name}, RegAt<0, 2>, RegAtValue<3, 5>, ThumbImm5ExtendedTo6, PreIncrement>"
    elif name in ["str", "ldr"] and subname == "imm5":
        op_name = name.capitalize()
        return f"thumb::thumb_single_data_transfer::<{op_name}, RegAt<0, 2>, RegAtValue<3, 5>, ThumbImm5ExtendedTo7, PreIncrement>"
    elif name in ["ldrsp", "strsp"]:
        op_name = (name[:-2]).capitalize()
        register = int(subname[1:])
        return f"thumb::thumb_single_data_transfer::<{op_name}, RegAt<8, 9>, RegValue<13>, ThumbImm8ExtendedTo10, PreIncrement>"
    elif name == "addpc":
        register = int(subname[1:])
        return f"thumb::thumb_load_address::<{register}, WordAlignedPc>"
    elif name == "addsp" and subname.startswith("r"):
        register = int(subname[1:])
        return f"thumb::thumb_load_address::<{register}, RegValue<13>>"
    elif name == "addsp" and subname == "imm7":
        return f"thumb::thumb_add_sp"
    elif name == "push":
        rlist = "ThumbRegisterListWithLr" if subname == "lr" else "ThumbRegisterList"
        return f"thumb::thumb_block_data_transfer::<Stm, ConstReg<13>, {rlist}, PreDecrement>"
    elif name == "pop":
        rlist = "ThumbRegisterListWithPc" if subname == "pc" else "ThumbRegisterList"
        return f"thumb::thumb_block_data_transfer::<Ldm, ConstReg<13>, {rlist}, PostIncrement>"
    elif name == "stmia":
        register = int(subname[1:])
        return f"thumb::thumb_block_data_transfer::<Stm, ConstReg<{register}>, ThumbRegisterList, PostIncrement>"
    elif name == "ldmia":
        register = int(subname[1:])
        return f"thumb::thumb_block_data_transfer::<Ldm, ConstReg<{register}>, ThumbRegisterList, PostIncrement>"
    elif name in [
        "beq",
        "bne",
        "bcs",
        "bcc",
        "bmi",
        "bpl",
        "bvs",
        "bvc",
        "bhi",
        "bls",
        "bge",
        "blt",
        "bgt",
        "ble",
    ]:
        cond = "COND_" + name[1:].upper()
        return f"thumb::thumb_conditional_branch::<{cond}>"
    elif name == "b":
        return f"thumb::thumb_unconditional_branch"
    elif name == "swi":
        return "thumb::thumb_swi"
    elif _class == "und":
        return "thumb::thumb_undefined"

    print(
        f"unknown THUMB instruction {name}/{subname} -- {desc} -- {subdesc} -- {_class}"
    )

    return "thumb::todo"


INSTRUCTION_TABLE_DIR = "scripts/data"
OUTPUT_FILE = "arm/src/lookup.rs"

DATAPROC_INSTR = [
    "adc",
    "add",
    "and",
    "bic",
    "cmn",
    "cmp",
    "eor",
    "mov",
    "mvn",
    "orr",
    "rsb",
    "rsc",
    "sbc",
    "sub",
    "teq",
    "tst",
]
DATAPROC_INSTR_S = [x + "s" for x in DATAPROC_INSTR]


def main():
    path = os.path.join(INSTRUCTION_TABLE_DIR, "lut-header.rs")
    file = open(path, "r")
    lut_code = file.read()
    file.close()

    path = os.path.join(INSTRUCTION_TABLE_DIR, "arm-instruction-table.json")
    file = open(path, "r")
    arm_data = json.load(file)
    file.close()
    arm_lut = generate_arm_lut(arm_data)
    lut_code += generate_lut_code("ARM_OPCODE_TABLE", arm_lut)

    path = os.path.join(INSTRUCTION_TABLE_DIR, "thumb-instruction-table.json")
    file = open(path, "r")
    thumb_data = json.load(file)
    file.close()
    thumb_lut = generate_thumb_lut(thumb_data)
    lut_code += generate_lut_code("THUMB_OPCODE_TABLE", thumb_lut)

    path = open(OUTPUT_FILE, "w")
    path.write(lut_code)
    path.close()

    return


def generate_lut_code(name, lut):
    instr_per_line = 1
    count = len(lut)
    current_instr_on_line = 0

    s = f"\n#[rustfmt::skip]\npub const {name}: [InstrFn; {count}] = ["
    if len(lut) > 0:
        s += "\n"
        for entry in lut:
            if current_instr_on_line >= instr_per_line:
                s += "\n"
                current_instr_on_line = 0

            if current_instr_on_line == 0:
                s += "    "
            else:
                s += " "

            s += f"{entry},"
            current_instr_on_line += 1
        if current_instr_on_line > 0:
            s += "\n"
    s += "];\n"
    return s


def generate_arm_lut(data):
    lut = []
    for row in data:
        for col in row:
            entry = arm_instr_data_to_lut_entry(col)
            lut.append(entry)
    return lut


def generate_thumb_lut(data):
    lut = []
    for row in data:
        for col in row:
            entry = thumb_instr_data_to_lut_entry(col)
            lut.append(entry)
    return lut


if __name__ == "__main__":
    main()
