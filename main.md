- what if we interpreted the wasm directly?
    - then we would need no bytecode and no map from bytecode to wasm
      to work with dwarf & stuff.
    - branches would be tricky, and br-table would be a huge issue.
      but we could have a hashmap from pc (index) to helper data.
      like br-table size - oh actually, we don't care how large br-table is,
      cause it always jumps somewhere.
      so yeah, we'd just have some hashmaps to accelerate jumps (and call/ret).
    - for impl, i'd say, completely get rid of `Compiler` & make those
      hashmaps optional outputs of the validator (along with the immutable stack).
- fix compiler size computation.
    - we're ignoring the pop in `pop,push`.
    - i think the immutable stack might be the way to go.
      for lua.wasm, it seems to be ~500k of data.
      for just the stack with parent pointers.
      we'd probably want an acceleration structure to go from (now wasm) pc
      to stack top index. which again, could just be a hashmap.
      so we're talking `#ops * (2*4) * 16/14`.
      this hashmap could also be used to validate that an offset is a valid
      opcode, for setting breakpoints.

- rw#2:
    - table driven validator.
        - generate opcode class table.
        - generate push/pop tables.
        - class dependent validation.
        - generate unreachable table.
    - immutable type stack.
    - optionally generate pc -> type top & pc -> br hash-maps.
    - wasm interp.

- todo:
    - unified stack.
        - 4 bit sp offset from frame.
        - account for `StackFrame` in bounds check.
        - locals, select.
            - careful with uninit.
            - ig we wanna switch on the align?
            - thinking just impl on stack view.
        - push frame needs to copy over params.
          can init locals while it's at it.
        - use it.
        - pop frame.
    - debugging.


- backlog:
    - exhaustion tests.
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
    - consider leb128 or other compression for bytecode operands.
    - consider a type table (for optimized callindirect).
    - mutable instances.
        - generational indices.
    - should we do an immutable stack?
        - could use that for debugging, no need to compute types.
    - a missing import polymorphic host function,
      error or default values or ask for values.

- robustness:
    - make sure validator/compiler begin/end* are called.
    - traps & host funcs.
    - is there a `loop.params-shifter`-like validation test with an invalid push (type) in the loop?
    - static assert little endian for ctype?
    - test multi-level func vars, occurs check.
    - test `caller_memory` `CallerNoMemory` error.
    - host fn panics (abort).

- cleanup:
    - we should probably have some abstraction for the unsafe cell thing.
    - `Memory::bounds_check`.
    - getrev, rw manual vec.
    - reader expectn: Err(eof: bool).

- sti:
    - `reserve_extra` retry with exact value on failure?
    - `reserve_extra` `assume!(self.cap >= min_cap)`.


