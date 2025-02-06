# wlug
`wlug` is a WASM plugin system that enables easy embedding of WASM plugins in your app, allowing seamless interaction between plugins and your host application.

## Examples
The [`embed`](https://github.com/serd223/wlug/tree/master/examples/embed.rs) example in the examples directory is an example that loads and executes the plugins inside [`examples/plugs/`](https://github.com/serd223/wlug/tree/master/examples/plugs) while also exporting some 'host functions' for the plugins to use.
Plugins 1-4 are written in Rust and `plug5` is written in C.

### Build instructions for the examples
### Prerequisites
#### Windows
- [Rust](https://www.rust-lang.org/tools/install)
- wasm32-unknown-unknown target for Rust: `rustup target add wasm32-unknown-unknown`
- [clang](https://releases.llvm.org/download.html)

#### Linux
- [Rust](https://www.rust-lang.org/tools/install)
- wasm32-unknown-unknown target for Rust: `rustup target add wasm32-unknown-unknown`
- [clang](https://releases.llvm.org/download.html) (compiling to wasm might require extra llvm tools depending on your distro and configuration)


### Instructions
- The instructions below assume you are in the root of the repository
- The instructions below build and run the `embed` example

#### Windows
```console
  $ ./build_plugs.ps1
  $ cargo run --example embed
```

#### Linux
```console
  $ ./build_plugs.sh
  $ cargo run --example embed
```


## Plugin structure
Each plugin consists of a single WASM module that is loaded dynamically by the `Plugs::load` interface. Each plugin can define its own functions and interact with other plugins and the host.

## Special exports
Metadata about plugins are communicated through special reserved exports. The actual names for these functions can be customized with the `Plugs::with_*` family of methods. The names below are the default ones you get with a `Plugs` instance created with `Plugs::new`.

### __name
Plugins need to export a special `__name` function to export their name as a null-terminated string. Each plugin needs to have a unique name and the `Plugs::load_module` function will return an error if it encounters a plugin name that already exists.
Here is the signature of the required `__name` function:
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

### __deps
Plugins can optionally export a `__deps` function which returns a null-terminated string that contains the list of plugins they depend on seperated by semicolons (';'). 

The plugin names used in this functions are the same names that plugins export with their `__name` function.
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
The names you supply in this function are the same names that are exported by the plugin's `__name`. Then you can forward declare any functions you want to use from the other plugin.

During linkage, `Plugs::link` looks for a plugin's unknown imports inside the dependencies exported by `__deps`. The order of which these dependencies are imported is also important. If two dependencies export a function with the same name and the dependent wants to import this function, only the function from the dependency that was declared earlier in the list will be imported.

### __init
`Plugs::init` executes each plugin's `__init` function. `Plugs::init` is typically called right after `Plugs::link` and before any `call` operations.
A common use case for this function is to initialize memory for state management in WASM memory. (See [`plug1`](https://github.com/serd223/wlug/blob/master/examples/plugs/plug1/src/lib.rs)) 
```rs
// Rust
#[no_mangle]
pub extern "C" fn __init() {
    // do init
}
```
```c
// C
void __init() {
    // do init
}
```

## Host functions

The host (your app) can declare 'host functions' and add them to the `Plugs` with `add_host_fn` to expose necessary functionality to plugins. In addition to the arguements passed by the plugin, host functions also have access to the caller plugin's exports (like its `memory`), generic user defined state and the unique id for the caller plugin. These functions allow plugins and the host to communicate and have shared state.

Plugins can forward declare these host functions and use them like normal. All you need to do is call `add_host_fn` in the host application to add your host functions and `Plugs` will handle the necessary linking.

For examples on using host functions, see the [`embed`](https://github.com/serd223/wlug/tree/master/examples/embed.rs) example.
