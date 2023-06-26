import json
import os


def arm_instr_data_to_lut_entry(data):
    name = data["name"].lower()
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

    if name in ["ldr", "str", "ldrb", "strb", "ldrt", "strt", "ldrbt", "strbt"]:
        op_name = name.capitalize()

        bindex = op_name.rfind("b")
        if bindex > 2:
            op_name = op_name[0:bindex] + op_name[(bindex + 1) :] + "B"
        tindex = op_name.rfind("t")
        if tindex > 2:
            op_name = op_name[0:tindex] + op_name[(tindex + 1) :]
            op_name = f"{op_name}<FORCE_USER_MODE>"

        writeback = "false"

        if "post-increment" in subdesc:
            indexing = "PostIncrement"
            writeback = "true"

        elif "pre-increment" in subdesc:
            indexing = "PreIncrement"
            writeback = "true"
        elif "positive" in subdesc:
            indexing = "PreIncrement"

        elif "post-decrement" in subdesc:
            indexing = "PostDecrement"
            writeback = "true"

        elif "pre-decrement" in subdesc:
            indexing = "PreDecrement"
            writeback = "true"
        elif "negative" in subdesc:
            indexing = "PreDecrement"

        else:
            indexing = ""
            print("indexing not found: " + subname)

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

        return f"arm::arm_single_data_transfer::<{op_name}, {offset}, {indexing}, {writeback}>"

    if name == "blx":
        return "arm::blx"
    if name == "bkpt":
        return "arm::bkpt"
    if name == "clz":
        return "arm::clz"

    if _class == "und":
        return "arm::undefined"
    if _class == "edsp":
        return "arm::m_extension_undefined"

    print(f"unknown instruction {name}/{subname}")
    return "arm::todo"


def thumb_instr_data_to_lut_entry(data):
    name = data["name"].lower()
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

    s = f"\npub const {name}: [InstrFn; {count}] = ["
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
