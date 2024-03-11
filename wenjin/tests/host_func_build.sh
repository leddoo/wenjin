clang \
    --target=wasm32 \
    -O3 \
    -Wl,--export=fib \
    -nostdlib \
    -o host_func.wasm \
    host_func.c

