
- todo:
    - compiler cleanup:
        - remove frames.
        - remove oom.
        - should we do an immutable stack?
            - could use that for debugging, no need to compute types.
    - unified stack.
        - stack compression.
    - put missing end logic into validator (end function).
    - debugging.


- backlog:
    - exhaustion tests.
    - no oom/invalid handle?
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
    - consider leb128 or other compression for bytecode operands.
    - consider a type table (for optimized callindirect).
    - mutable instances.
        - generational indices.

- robustness:
    - traps & host funcs.
    - is there a `loop.params-shifter`-like validation test with an invalid push (type) in the loop?
    - static assert little endian for ctype?
    - test multi-level func vars, occurs check.
    - test `caller_memory` `CallerNoMemory` error.
    - host fn panics (abort).

- cleanup:
    - we should probably have some abstraction for the unsafe cell thing.
    - `Memory::bounds_check`.
    - getrev, rw manual vec.
    - reader expectn: Err(eof: bool).

- sti:
    - `reserve_extra` retry with exact value on failure?
    - `reserve_extra` `assume!(self.cap >= min_cap)`.


