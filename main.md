
- todo:
    - unified stack.
        - compression.
            - compute max local/push size.
        - consider extracting stack logic.
            - so the Op -> push/pop type sequence.
              validator would use internal stack to ensure integrity.
              compiler would keep track of max operand size.
            - we could also do stack polymorphism while we're at it.
            - that thing could also (help) keep track of the unreachable state.
            - how would we do that?
                - easiest thing would be create Operator & switch multiple times.
                - so we still want the validator and compiler to have separate state.
                  cause they're in different crates.
                  we want distinct functions push/pop on validator & compiler,
                  which are supposed to be called automatically.
                - we could extend the operatorvisitor trait.
                    - `visit_op/core`, `push`, `pop`, `unreachable`.
                    - and you just override the `visit_op_core` method.
                    - or we put the driver logic into a separate generic fn.
                    - let's try that.
        - put missing end logic into validator (end function).
    - exhaustion tests.
    - clone the wasm (cause we have dangling refs).
    - debugging.
    - mutable instances.
        - generational indices.


- backlog:
    - validation:
        - duplicate exports.
        - memory <= 4gib.
        - data count section.
    - table get/set.
    - caller-global?
    - typed global api.
    - wat parser.
    - make ids wasmtype/ctype.
    - granular mem string utils, tests.
    - proper parse error sources.
    - div & conversion traps.

- robustness:
    - traps & host funcs.
    - is there a `loop.params-shifter`-like validation test with an invalid push (type) in the loop?
    - static assert little endian for ctype?
    - test multi-level func vars, occurs check.
    - test `caller_memory` `CallerNoMemory` error.
    - host fn panics (abort).

- cleanup:
    - `Memory::bounds_check`.
    - getrev, rw manual vec.
    - reader expectn: Err(eof: bool).

- sti:
    - `reserve_extra` retry with exact value on failure?
    - `reserve_extra` `assume!(self.cap >= min_cap)`.


