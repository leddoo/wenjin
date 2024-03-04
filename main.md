
- todo:
    - rewrite:
        - interp vm.
            - just one stack for now, we can do threads later.
            - stack is shared resource, may realloc, need to update ptr after opaque call.
        - call dyn.
        - wasm types & typed func.
        - wasm test suite.
        - rest of the owl:
            - globals.
            - data.
            - tables.
            - elements.
            - `br_table`.
    - merge interp stuff into `Store`.
    - `MemoryCtx`.
        - basically just a pointer to `StoreMemory`.
        - requires `&mut Store`.
        - may invalidate interp mem ptr.
            - so interp state must make sure that it fixes that ptr
              on return of host fn.
            - maybe have some shared "dirty flag" as an opt.
              but for now, prefer deopt & simplify.
        - move memory stuff into separate file.
    - get rid of /para
    - new `Ctx` api.
        - document where stuff is stored,
          and when it is aliased by what.
        - put all the common functions on `Ctx`.
            - `Store: Deref<Ctx>`.
            - wait no, just give access to `Store`.
            - and make sure the aliasing invariants are upheld.
        - `MemoryCtx`.
            - heap allocate memories.
            - pointer.
            - consider getting rid of the lifetime.
    - globals api.
        - `Global` and `DynGlobal`. (rename func)
    - object refs.
        - special global/func/memory/etc that can be null or id.
    - mimalloc it up.
        - crud wasm objects.
        - gen indices (could have store specific init value).
        - when trying to delete running functions, defer.
    - impl unimpls.
    - parser trait instead of the inline thing.
    - robustness.

