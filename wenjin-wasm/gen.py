from opcodes import *

def to_upper_camel(name):
    return "".join(map(lambda part: part.capitalize(), name.split("_")))


def opcode_enum():
    print("enum Opcode {")
    for opcode in opcodes:
        name = to_upper_camel(opcode[COL_NAME])
        print(f"    {name},")
    print("}")


def parse_table():
    pass


opcode_enum()
parse_table()



