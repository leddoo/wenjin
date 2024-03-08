
- todo:
    - pass all (`invoke`/`assert_return`) tests.
        - little endian.
            - memory stuff `to_le_bytes`.
            - for `CType`, could use trait.
              but maybe we just static-assert little endian.
        - memory grow/size.
        - fp nearest.
    - host funcs.
    - wasm types & typed func.
    - rest of the owl:
        - clone the wasm (cause we have dangling refs).
        - validate non-code sections (incl elem, data).
        - pass remaining tests.
    - debugging.


- robustness:
    - is there a `loop.params-shifter`-like validation test with an invalid push (type) in the loop?

- sti:
    - `reserve_extra` retry with exact value on failure?
    - `reserve_extra` `assume!(self.cap >= min_cap)`.


