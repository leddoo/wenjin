import re

from opcodes import *


def to_upper_camel(name):
    return "".join(map(lambda part: part.capitalize(), name.split("_")))


def opcode_enum():
    print("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]")
    print("#[repr(u8)]")
    print("pub enum Opcode {")
    for opcode in opcodes:
        name = to_upper_camel(opcode[COL_NAME])
        print(f"    {name},")
    print("}")
    print(f"const NUM_OPCODES: usize = {len(opcodes)};")


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

    print("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]")
    print("#[repr(u8)]")
    print("pub enum Prefix {")
    for k, v in sorted(tab.items()):
        if isinstance(v, dict):
            print(f"    X{k:x},")
    print("}")

    print("const PARSE: &[ParseResult; 256] = &[")
    for b in range(256):
        v = tab.get(b)
        if v:
            if isinstance(v, str):
                print(f"    ParseResult::Opcode(Opcode::{to_upper_camel(v)}),")
            else:
                print(f"    ParseResult::Prefix(Prefix::X{k:x}),")
        else:
            print("    ParseResult::Error,")
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
    print("fn parse_prefixed_core(prefix: Prefix, v: u32) -> Option<Opcode> {")
    print("    match prefix {")
    for k, v in sorted(tab.items()):
        if not isinstance(v, dict):
            continue
        print(f"        Prefix::X{k:x} => parse_x{k:x}(v),")
    print("    }")
    print("}")


def opcode_class():
    print("#[derive(Clone, Copy, Debug)]")
    print("pub enum OpcodeClass {")
    print("    Basic { pop: &'static [ValueType], push: &'static [ValueType] },")
    print("    Mem { max_align: u8, pop: &'static [ValueType], push: &'static [ValueType] },")
    for opcode in opcodes:
        name = opcode[COL_NAME]
        imm = opcode[COL_IMM]
        args = opcode[COL_ARGS]
        rets = opcode[COL_RETS]
        flags = opcode[COL_FLAGS]

        special = "c" in flags
        assert not len(imm) > 0 or special or "m" in flags
        assert not "!" in flags or special
        assert not special or len(args) == 0
        assert not special or len(rets) == 0

        if special:
            print(f"    {to_upper_camel(name)},")
    print("}")

    print("const CLASS: &[OpcodeClass; NUM_OPCODES] = &[")
    for opcode in opcodes:
        name = opcode[COL_NAME]
        args = opcode[COL_ARGS]
        rets = opcode[COL_RETS]
        flags = opcode[COL_FLAGS]

        special = "c" in flags
        if special:
            print(f"    OpcodeClass::{to_upper_camel(name)},")
        else:
            pop  = ",".join(map(lambda ty: f"ValueType::{ty.capitalize()}", args))
            push = ",".join(map(lambda ty: f"ValueType::{ty.capitalize()}", rets))
            if "m" in flags:
                m = re.findall(r"\d+", name)
                assert m
                assert len(m) == 1 or len(m) == 2
                max_align = int(m[-1])//8
                print(f"    OpcodeClass::Mem {{ max_align: {max_align}, pop: &[{pop}], push: &[{push}] }},")
            else:
                print(f"    OpcodeClass::Basic {{ pop: &[{pop}], push: &[{push}] }},")
    print("];")


opcode_enum()
parse_table()
opcode_class()


