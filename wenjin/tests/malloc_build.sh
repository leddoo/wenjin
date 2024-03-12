clang \
    --target=wasm32 \
    -O3 \
    -Wl,--export=run \
    -nostdlib \
    -o malloc.wasm \
    malloc.c

