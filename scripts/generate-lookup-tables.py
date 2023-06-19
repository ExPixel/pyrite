import json
import os


INSTRUCTION_TABLE_DIR = "scripts/data"
OUTPUT_FILE = "arm/src/lookup.rs"


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


def arm_instr_data_to_lut_entry(data):
    name = data["name"].lower()
    return "arm::todo"


def generate_thumb_lut(data):
    lut = []
    for row in data:
        for col in row:
            entry = thumb_instr_data_to_lut_entry(col)
            lut.append(entry)
    return lut


def thumb_instr_data_to_lut_entry(data):
    name = data["name"].lower()
    return "thumb::todo"


if __name__ == "__main__":
    main()
