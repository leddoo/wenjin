
- todo:
    - pass all (`invoke`/`assert_return`) tests.
        - tables. & call indirect.
        - globals.
        - data.
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


