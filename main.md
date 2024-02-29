
- todo:
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
    - robustness.

