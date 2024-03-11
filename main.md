
- todo:
    - imports.
    - interp to host calls.
    - rest of the owl:
        - clone the wasm (cause we have dangling refs).
        - validate non-code sections (incl elem, data).
        - wasm ptr.
        - ctype.
        - pass remaining tests.
    - debugging.
    - mutable instances.
        - generational indices.


- backlog:
    - typed global api.

- robustness:
    - is there a `loop.params-shifter`-like validation test with an invalid push (type) in the loop?
    - static assert little endian for ctype?

- sti:
    - `reserve_extra` retry with exact value on failure?
    - `reserve_extra` `assume!(self.cap >= min_cap)`.


