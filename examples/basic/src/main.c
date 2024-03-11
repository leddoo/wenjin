void __wasm_call_ctors();
void _start() { __wasm_call_ctors(); }


int add(int a, int b) {
    return a + b;
}

float thing(float init, int n) {
    float result = init;
    for(int i = 0; i < n; i += 1) {
        if(i % 2) {
            result += 1.0f;
        }
        else {
            result *= 2.0f;
        }
    }
    return result;
}

int fib(int n) {
    if(n < 2) {
        return n;
    }
    else {
        return fib(n-2) + fib(n-1);
    }
}

