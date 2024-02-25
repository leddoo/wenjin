void __wasm_call_ctors();

void _start() {
    __wasm_call_ctors();
}


int add(int a, int b) {
    return a + b;
}

int fib(int n) {
    if(n < 2) {
        return n;
    }
    else {
        return fib(n-2) + fib(n-1);
    }
}

