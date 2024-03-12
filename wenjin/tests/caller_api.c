void __wasm_call_ctors();
void _start() { __wasm_call_ctors(); }


__attribute__((import_module("host"), import_name("host_func")))
extern void host_func(int n);

void call_host(int n) {
    host_func(n);
}

