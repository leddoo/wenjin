
- todo:
    - validation tests.
    - validate non-code sections (incl elem, data).
        - (debug) asserts in store.
    - table get/set.
    - unified stack.
    - getrev, rw manual vec.
    - global api.
        - `caller_global(name)`?
    - rest of the owl:
        - clone the wasm (cause we have dangling refs).
        - non-func imports.
        - pass remaining tests.
    - debugging.
    - mutable instances.
        - generational indices.


- backlog:
    - wasm-testsuite imports.
    - typed global api.
    - wat parser.
    - make ids wasmtype/ctype.
    - granular mem string utils, tests.
    - proper parse error sources.

- robustness:
    - is there a `loop.params-shifter`-like validation test with an invalid push (type) in the loop?
    - static assert little endian for ctype?
    - test multi-level func vars, occurs check.
    - test `caller_memory` `CallerNoMemory` error.
    - host fn panics (abort).

- cleanup:
    - `Memory::bounds_check`.

- sti:
    - `reserve_extra` retry with exact value on failure?
    - `reserve_extra` `assume!(self.cap >= min_cap)`.


