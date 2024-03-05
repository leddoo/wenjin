
- todo:
    - pass all tests.
        - `br_table`.
        - memory ops.
    - host funcs.
    - wasm types & typed func.
    - rest of the owl:
        - globals.
        - data.
        - tables.
        - elements.
        - remaining ops.
        - clone the wasm (cause we have dangling refs).
        - validate non-code sections (incl elem, data).

- bugs:
    - block return types. (need to generate `return`-like instruction)


