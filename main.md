
- todo:
    - global api.
        - `caller_global(name)`?
    - panic eh?
    - rest of the owl:
        - clone the wasm (cause we have dangling refs).
        - validate non-code sections (incl elem, data).
        - non-func imports.
        - pass remaining tests.
    - debugging.
    - mutable instances.
        - generational indices.


- backlog:
    - typed global api.
    - wat parser.
    - make ids wasmtype/ctype.
    - granular mem string utils, tests.

- robustness:
    - is there a `loop.params-shifter`-like validation test with an invalid push (type) in the loop?
    - static assert little endian for ctype?
    - test multi-level func vars, occurs check.
    - test `caller_memory` `CallerNoMemory` error.
    - host fn panics.

- sti:
    - `reserve_extra` retry with exact value on failure?
    - `reserve_extra` `assume!(self.cap >= min_cap)`.


