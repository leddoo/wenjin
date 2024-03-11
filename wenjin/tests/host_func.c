void __wasm_call_ctors();
void _start() { __wasm_call_ctors(); }


__attribute__((import_module("host"), import_name("fib_host")))
extern int fib_host(int n);

int fib(int n) {
    if(n < 2) {
        return n;
    }
    else {
        return fib_host(n-2) + fib_host(n-1);
    }
}

