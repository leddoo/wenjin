
def gen(n: int):
    result = ""

    type_decls = ", ".join(map(lambda i: f"T{i}: WasmType", range(n)))
    types      = ", ".join(map(lambda i: f"T{i}", range(n))) + ","
    names      = ", ".join(map(lambda i: f"a{i}", range(n)))


    # `impl WasmTypes`
    result += f"    impl<{type_decls}> WasmTypes for ({types}) {{\n"

    # `const WASM_TYPES`
    result += "        const WASM_TYPES: &'static [wasm::ValueType] = &["
    result += ", ".join(map(lambda i: f"T{i}::WASM_TYPE", range(n)))
    result += "];\n\n"

    # `fn to_stack_values`
    result += "        #[inline(always)]\n"
    result += "        unsafe fn to_stack_values(self, dst: *mut StackValue) { unsafe {\n"
    for i in range(n):
        result += f"            dst.add({i}).write(self.{i}.to_stack_value());\n"
    result += "        }}\n\n"

    # `fn from_stack_values`
    result += "        #[inline(always)]\n"
    result += "        unsafe fn from_stack_values(src: *const StackValue) -> Self { unsafe { (\n"
    for i in range(n):
        result += f"            T{i}::from_stack_value(src.add({i}).read()),\n"
    result += "        )}}\n"

    result += "    }\n\n"


    # `impl HostFunc for Fn(Ts) -> Rs`
    result += f"    unsafe impl<{type_decls}, R: WasmResult, F: Fn({types}) -> R + 'static> HostFunc<({types}), R::Types, false> for F {{\n"
    result +=  "        #[inline]\n"
    result +=  "        fn call(&self, store: &mut Store) -> Result<(), Error> {\n"
    result +=  "            let stack = &mut store.thread.stack;\n"
    result += f"            unsafe {{ stack.set_len(stack.len() - {n}) }};\n"
    result += f"            let ({names},) = unsafe {{ WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) }};\n"
    result += f"            let r = (self)({names}).to_result()?;\n"
    result += f"            let stack = &mut store.thread.stack;\n"
    result +=  "            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };\n"
    result +=  "            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };\n"
    result +=  "            Ok(())\n"
    result +=  "        }\n"
    result +=  "    }\n\n"

    # `impl HostFunc for Fn(&mut store, Ts) -> Rs`
    result += f"    unsafe impl<{type_decls}, R: WasmResult, F: Fn(&mut Store, {types}) -> R + 'static> HostFunc<({types}), R::Types, true> for F {{\n"
    result +=  "        #[inline]\n"
    result +=  "        fn call(&self, store: &mut Store) -> Result<(), Error> {\n"
    result +=  "            let stack = &mut store.thread.stack;\n"
    result += f"            unsafe {{ stack.set_len(stack.len() - {n}) }};\n"
    result += f"            let ({names},) = unsafe {{ WasmTypes::from_stack_values(stack.as_mut_ptr().add(stack.len())) }};\n"
    result += f"            let r = (self)(store, {names}).to_result()?;\n"
    result += f"            let stack = &mut store.thread.stack;\n"
    result +=  "            unsafe { r.to_stack_values(stack.as_mut_ptr().add(stack.len())) };\n"
    result +=  "            unsafe { stack.set_len(stack.len() + R::Types::WASM_TYPES.len()) };\n"
    result +=  "            Ok(())\n"
    result +=  "        }\n"
    result +=  "    }\n\n"

    print(result)



for i in range(1, 16 + 1):
    gen(i)

