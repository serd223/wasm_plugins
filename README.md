# Simple WASM Plugin System
A plugin system I wrote to use in my own apps where I need a simple plugin system. 

## Repository structure
The `embed` crate is working binary example of the system in action. The `wasm_plugs` crate is the library that the `embed` example imports to use the system.

The `plugs` directory contains the source code of 5 example plugins. Plugins 1-4 are written in Rust and `plug5` is written in C.

## Plugin structure
Each plugin consists of a single wasm module.

Plugins need to export a special `__name` function for their name. Each plugin needs to have a unique name and the `load_module` function will return an error if it encounters a plugin name that already exists. 
Here is there signature of the required `__name` function:
```rs
// Rust
#[no_mangle]
pub extern "C" fn __name() -> *const u8 {
    b"plug1\0".as_ptr()
}
```
```c
// C
const char* __name() {
    return "plug1";
}
```

The host (your app) can declare 'host functions' and add them to the `Plugs` with `add_host_fn` to expose necessary functionality to plugins. In addition to the arguements passed by the plugin, host functions also have access to the caller plugin's exports (like its `memory`), generic user defined state and the unique id for the caller plugin.

Plugins can forward declare these host functions and use them like normal. All you need to do is call `add_host_fn` in the host application to add your host functions and `Plugs` will handle the necessary linking.

There are some example host functions declared in `embed/src/main.rs`. (see the example plugins for their usage)

In order to import from another plugin, you first need to export a special function named `__deps` with the following signature:
```rs
// Rust
#[no_mangle]
pub extern "C" fn __deps() -> *const u8 {
    b"plug1;plug2\0".as_ptr()
}
```
```c
// C
const char* __deps() {
    return "plug1;plug2";
}
```
Then you can forward declare any functions you want to use from the other plugin just like you can with the host functions. The names you supply in this function are the same names that are exported by the plugin's `__name`.

## Build instructions for the `embed` example
## Prerequisites
### Windows
- [Rust](https://www.rust-lang.org/tools/install)
- wasm32-unknown-unknown target for Rust: `rustup target add wasm32-unknown-unknown`
- [clang](https://releases.llvm.org/download.html)

### Linux
- [Rust](https://www.rust-lang.org/tools/install)
- wasm32-unknown-unknown target for Rust: `rustup target add wasm32-unknown-unknown`
- [clang](https://releases.llvm.org/download.html) (compiling to wasm might require extra llvm tools depending on your distro and configuration)


## Instructions
- The instructions below assume you are in the root of the repository

Windows:
```console
  $ ./build_plugs.ps1
  $ cd embed
  $ cargo run
```

Linux:
```console
  $ ./build_plugs.sh
  $ cd ./embed
  $ cargo run
```

