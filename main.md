
- todo:
    - pass all (`invoke`/`assert_return`) tests.
        - fix block return types.
            - adjust frame height using params.
            - store num block rets.
            - actually, we need the info on the instrs,
              cause `br_if` only needs the shift on break.
              so let's have special versions of those as two-byte opcodes.
              return is probably common, so maybe don't have a special version of that.
              but tbh, let's just not have specials for now.
              again, it's the interp, who cares.
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


