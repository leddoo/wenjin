
- todo:
    - remove modules.
    - unified stack.
        - compression.
            - compute max local/push size.
            - make compiler own validator.
        - put missing end logic into validator (end function).
    - consider leb128 or other compression for bytecode operands.
    - consider a type table (for optimized callindirect).
    - exhaustion tests.
    - clone the wasm (cause we have dangling refs).
    - debugging.
    - mutable instances.
        - generational indices.


- backlog:
    - validation:
        - duplicate exports.
        - memory <= 4gib.
        - data count section.
    - table get/set.
    - caller-global?
    - typed global api.
    - wat parser.
    - make ids wasmtype/ctype.
    - granular mem string utils, tests.
    - proper parse error sources.
    - div & conversion traps.

- robustness:
    - traps & host funcs.
    - is there a `loop.params-shifter`-like validation test with an invalid push (type) in the loop?
    - static assert little endian for ctype?
    - test multi-level func vars, occurs check.
    - test `caller_memory` `CallerNoMemory` error.
    - host fn panics (abort).

- cleanup:
    - `Memory::bounds_check`.
    - getrev, rw manual vec.
    - reader expectn: Err(eof: bool).

- sti:
    - `reserve_extra` retry with exact value on failure?
    - `reserve_extra` `assume!(self.cap >= min_cap)`.


