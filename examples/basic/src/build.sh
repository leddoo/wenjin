clang \
    --target=wasm32 \
    -O3 \
    -Wl,--export=add \
    -Wl,--export=thing \
    -Wl,--export=fib \
    -nostdlib \
    -o lib.wasm \
    main.c

