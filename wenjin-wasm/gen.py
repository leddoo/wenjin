from opcodes import *

def to_upper_camel(name):
    return "".join(map(lambda part: part.capitalize(), name.split("_")))


def opcode_enum():
    print("#[repr(u8)]")
    print("enum Opcode {")
    for opcode in opcodes:
        name = to_upper_camel(opcode[COL_NAME])
        print(f"    {name},")
    print("}")
    print()


def parse_table():
    assert len(opcodes) < 0x8000

    tab = {}
    for opcode in opcodes:
        name = opcode[COL_NAME]
        bs = opcode[COL_BYTES]
        if len(bs) == 1:
            b = bs[0]
            assert b < 256
            assert b not in tab
            tab[b] = name
        else:
            assert len(bs) == 2
            prefix = bs[0]
            assert prefix < 256
            u32 = bs[1]
            if prefix not in tab:
                tab[prefix] = {}
            else:
                assert isinstance(tab[prefix], dict)
            tab[prefix][u32] = name

    print("#[repr(u8)]")
    print("enum Prefix {")
    for k, v in sorted(tab.items()):
        if isinstance(v, dict):
            print(f"    X{k:x},")
    print("}")

    print("const PARSE: &[u16; 256] = &[")
    for b in range(256):
        v = tab.get(b)
        if v:
            if isinstance(v, str):
                print(f"    Opcode::{to_upper_camel(v)} as u16,")
            else:
                print(f"    0x8000 | Prefix::X{k:x} as u16,")
        else:
            print("    0xffff,")
    print("];")

    for k, v in sorted(tab.items()):
        if not isinstance(v, dict):
            continue
        print("#[inline]")
        print(f"fn parse_x{k:x}(v: u32) -> Option<Opcode> {{")
        print("    Some(match v {")
        for k, v in sorted(v.items()):
            print(f"        {k} => Opcode::{to_upper_camel(v)},")
        print(f"        _ => return None")
        print("    })")
        print("}")

    print("#[inline]")
    print("fn parse_prefixed(prefix: Prefix, v: u32) -> Option<Opcode> {")
    print("    match prefix {")
    for k, v in sorted(tab.items()):
        if not isinstance(v, dict):
            continue
        print(f"        Prefix::X{k:x} => parse_x{k:x}(v),")
    print("    }")
    print("}")


def opcode_class():
    for opcode in opcodes:
        name = opcode[COL_NAME]
        imm = opcode[COL_IMM]
        flags = opcode[COL_FLAGS]
        if len(imm) > 0 or "c" in flags:
            print(name)
        else:
            print("#pure", name)
    pass


opcode_enum()
parse_table()
opcode_class()


