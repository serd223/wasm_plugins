#!/bin/sh
cd ./plug1
cargo build --release --target=wasm32-unknown-unknown
cp ./target/wasm32-unknown-unknown/release plug1.wasm ../
cd ..
cd ./plug2
cargo build --release --target=wasm32-unknown-unknown
cp ./target/wasm32-unknown-unknown/release plug2.wasm ../
cd ..
cd ./plug3
cargo build --release --target=wasm32-unknown-unknown
cp ./target/wasm32-unknown-unknown/release plug3.wasm ../
cd ..
cd ./plug4
cargo build --release --target=wasm32-unknown-unknown
cp ./target/wasm32-unknown-unknown/release plug4.wasm ../
cd ..
cd ./plug5
clang --target=wasm32 --no-standard-libraries -Wl,--export-all -Wl,--no-entry -Wl,--allow-undefined -o plug5.wasm plug5.c
cp ./plug5.wasm ../
cd ..
