import json
import os


def arm_instr_data_to_lut_entry(data):
    name = data["name"].lower()
    subname = data["subname"].lower()

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
            return f"arm::arm_dataproc::<{s_flag}, alu::{op_name}, alu::ImmOp2>"
        if subname == "lli":
            return f"arm::arm_dataproc::<{s_flag}, alu::{op_name}, alu::LliOp2>"
        if subname == "llr":
            return f"arm::arm_dataproc::<{s_flag}, alu::{op_name}, alu::LlrOp2>"
        if subname == "lri":
            return f"arm::arm_dataproc::<{s_flag}, alu::{op_name}, alu::LriOp2>"
        if subname == "lrr":
            return f"arm::arm_dataproc::<{s_flag}, alu::{op_name}, alu::LrrOp2>"
        if subname == "ari":
            return f"arm::arm_dataproc::<{s_flag}, alu::{op_name}, alu::AriOp2>"
        if subname == "arr":
            return f"arm::arm_dataproc::<{s_flag}, alu::{op_name}, alu::ArrOp2>"
        if subname == "rri":
            return f"arm::arm_dataproc::<{s_flag}, alu::{op_name}, alu::RriOp2>"
        if subname == "rrr":
            return f"arm::arm_dataproc::<{s_flag}, alu::{op_name}, alu::RrrOp2>"
        else:
            print("unknown dataproc subname: " + subname)

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
