cd examples\plugs\plug1
cargo build --release --target=wasm32-unknown-unknown
cp .\target\wasm32-unknown-unknown\release\plug1.wasm ..\..\..\
cd ..\..\..\
cd examples\plugs\plug2
cargo build --release --target=wasm32-unknown-unknown
cp .\target\wasm32-unknown-unknown\release\plug2.wasm ..\..\..\
cd ..\..\..\
cd examples\plugs\plug3
cargo build --release --target=wasm32-unknown-unknown
cp .\target\wasm32-unknown-unknown\release\plug3.wasm ..\..\..\
cd ..\..\..\
cd examples\plugs\plug4
cargo build --release --target=wasm32-unknown-unknown
cp .\target\wasm32-unknown-unknown\release\plug4.wasm ..\..\..\
cd ..\..\..\
cd examples\plugs\plug5
clang --target=wasm32 --no-standard-libraries -Wl','--export-all -Wl','--no-entry -Wl','--allow-undefined -o plug5.wasm plug5.c
cp .\plug5.wasm ..\..\..\
cd ..\..\..\


