
- todo:
    - pass all (`invoke`/`assert_return`) tests.
        - fix block return types.
            - shift params for branches.
        - `br_table`.
        - globals.
        - data.
        - tables.
        - call indirect.
        - elements.
        - memory ops.
        - remaining ops.
    - host funcs.
    - wasm types & typed func.
    - rest of the owl:
        - clone the wasm (cause we have dangling refs).
        - validate non-code sections (incl elem, data).
        - pass remaining tests.
    - debugging.


- robustness:
    - is there a `loop.params-shifter`-like validation test with an invalid push (type) in the loop?


