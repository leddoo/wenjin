clang \
    --target=wasm32 \
    -O3 \
    -Wl,--export=call_host \
    -nostdlib \
    -o caller_api.wasm \
    caller_api.c

