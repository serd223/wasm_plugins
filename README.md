# Simple WASM Plugin System
A plugin system I wrote to use in my own apps where I need a simple plugin system. 

## Repository structure
The `embed` crate is both a working binary example of the system in action and is also a library that contains the `Plugs` struct which is the main entry point of the system.

The `plugs` directory contains the source code of 5 example plugins. Plugins 1-4 are written in Rust and `plug5` is written in C.

## Plugin structure
Each plugin consists of a single .wasm file. The plugin's file name is used to refer to the plugin in code. (If the plugin's file name is `plug.wasm`, it will be referred to as `plug`)

There are some example 'host functions' exported by `embed/src/main.rs` which can be accessed by forward declaring them in your plugin source code. (see the example plugins)

In order to import from another plugin, you first need to export a special function named `deps` with the following signature:
```rs
// Rust
#[no_mangle]
pub extern "C" fn deps() -> *const u8 {
    b"plug1;plug2\0".as_ptr()
}
```
```c
// C
const char* deps() {
    return "plug1;plug2";
}
```
Then you can forward declare any functions you want to use from the other plugin just like you can with the host functions. The names you supply in this function are the same names that are inferred from their file names mentioned above.

## Build instructions for the `embed` example
## Prerequisites
### Windows
- Rust
- wasm32-unknown-unknown target for Rust: `rustup target add wasm32-unknown-unknown`

Note: Windows currently doesn't support compiling `plug5` because of my unwillingness to learn MSVC's linker options. You should comment out the lines which load and execute `plug5` in `embed/src/main.rs`.
### Linux
- Rust
- wasm32-unknown-unknown target for Rust: `rustup target add wasm32-unknown-unknown`
- clang (compiling to wasm might require extra llvm tools depending on your distro and configuration)


## Instructions
- The instructions below assume you are in the root of the repository

Windows:
```console
  $ ./build_plugs.ps1 # Wont build plug5
  $ cd embed
  $ cargo run
```

Linux:
```console
  $ ./build_plugs.sh
  $ cd ./embed
  $ cargo run
```

