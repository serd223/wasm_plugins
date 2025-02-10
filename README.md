# wlug
`wlug` is a WASM plugin system that lets you easily embed WASM plugins written in any language inside your app, also allowing seamless interaction between your application and the plugins.

This README contains build instructions for the examples and documentation on the basic structure of plugins.

## Using `wlug` in your projects
Currently `wlug` isn't on [crates.io](https://crates.io/) yet but you and add it to your project via this git repostiory by running the following in your project's root directory:
```console
    $ cargo add --git https://github.com/serd223/wlug wlug
```
Or you can add the following to the `dependencies` in your `Cargo.toml`:
```toml
wlug = { git = "https://github.com/serd223/wlug" }
```

## Examples
The [`embed`](https://github.com/serd223/wlug/tree/master/examples/embed.rs) example in the examples directory is an example that loads and executes the plugins inside [`examples/plugs/`](https://github.com/serd223/wlug/tree/master/examples/plugs) while also exporting some 'host functions' for the plugins to use.
Plugins 1-4 are written in Rust and `plug5` is written in C.

For more examples, see [wlug_examples](https://github.com/serd223/wlug_examples).

### Build instructions for the `embed` example
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

This export is optional and `Plugs::extract_metadata` will just skip calling a plugin's `__deps` if it doesn't export it.
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
Then plugins can forward declare any functions they want to use from the other plugin.

During linkage, `Plugs::link` looks for a plugin's unknown imports inside the dependencies exported by `__deps`. The order of which these dependencies are imported is also important. If two dependencies export a function with the same name and the dependent wants to import this function, only the function from the dependency that was declared earlier in the list will be imported.

### __init
`Plugs::init` executes each plugin's `__init` function. `Plugs::init` isn't automatically called and should typically be called right after `Plugs::link` and before any `call` operations.
A common use case for this function is to initialize memory in plugins for state management in WASM memory. (See [`plug1`](https://github.com/serd223/wlug/blob/master/examples/plugs/plug1/src/lib.rs)) 

This export is optional and `Plugs::init` will just skip calling a plugin's `__init` if it doesn't export it.
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

### __reset
Similar to `__init`, the `__reset` export of each plugin is called inside of `Plugs::reset`. This export can be used to handle state management between plugin reloads.

This export is optional and `Plugs::reset` will just skip calling a plugin's `__reset` if it doesn't export it.
```rs
// Rust
#[no_mangle]
pub extern "C" fn __reset() {
    // do reset
}
```
```c
// C
void __reset() {
    // do reset
}
```

## Host functions

The host (your app) can declare 'host functions' and add them to the `Plugs` with `add_host_fn` to expose necessary functionality to plugins. In addition to the arguements passed by the plugin, host functions also have access to the caller plugin's exports (like its `memory`), generic user defined state and the unique id for the caller plugin. These functions allow plugins and the host to communicate and have shared state.

Plugins can forward declare these host functions and use them like normal. All you need to do is call `add_host_fn` in the host application to add your host functions and `Plugs` will handle the necessary linking.

For examples on using host functions, see the [`embed`](https://github.com/serd223/wlug/tree/master/examples/embed.rs) example.
