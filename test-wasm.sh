#!/bin/bash

BUILD=true

for arg in "$@"; do
    if [[ "$arg" == "-skip" ]]; then
        BUILD=false
    fi
done

if [ "$BUILD" = true ]; then
    echo "Building WebAssembly packages..."
    wasm-pack build --target web --release --out-dir "./tests/wasm/bin/"
else
    echo "Skipping build..."
fi

PORT=8081
npx http-server "./tests/wasm" -p $PORT
