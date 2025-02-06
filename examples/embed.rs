use wasmtime::*;
use wlug::Plugs;

#[derive(Default)]
struct State {
    print_count: i32,
    print2_count: i32,
}

fn main() -> wasmtime::Result<()> {
    let engine = Engine::default();
    let my_state = State::default();
    let mut plugs = Plugs::new(&engine, my_state);
    // You can customize the expected export names with this builder-type API
    // The values below are the default values for these exports
    // .with_name("__name")
    // .with_deps("__deps")
    // .with_init("__init")

    plugs.add_host_fn("print".to_string(), my_core::print);
    plugs.add_host_fn("print2".to_string(), my_core::print2);

    // Load order is important and circular dependencies are disallowed
    plugs.load("plug1.wasm", &engine)?;
    plugs.load("plug2.wasm", &engine)?;
    plugs.load("plug3.wasm", &engine)?;
    plugs.load("plug4.wasm", &engine)?;
    plugs.load("plug5.wasm", &engine)?;

    for (name, plug) in plugs.items().iter() {
        println!("[INFO]: '{name}' metadata:");
        println!("[INFO]:     exports: {:?}", plug.exports);
        println!("[INFO]:     imports: {:?}\n", plug.imports);
    }

    println!("[INFO]: Starting to link...");
    plugs.link()?;
    println!("\n[INFO]: Linking is complete.\n");

    println!("[INFO]: Initializing...");
    plugs.init()?;
    println!("[INFO]: Initialization is complete.");

    println!("\n[INFO]: Calling plug1.plug1 with args: 10");
    plugs.call::<_, ()>("plug1", "plug1", 10i32)?;

    println!("\n[INFO]: Calling plug2.plug2 with args: 10");
    plugs.call::<_, ()>("plug2", "plug2", 10i32)?;

    println!("\n[INFO]: Calling plug3.plug3 with args: 10");
    plugs.call::<_, ()>("plug3", "plug3", 10i32)?;

    println!("\n[INFO]: Calling plug4.plug4 with args: 10");
    plugs.call::<_, ()>("plug4", "plug4", 10i32)?;

    println!("\n[INFO]: Calling plug5.hello_from_c with args: (10, 20)");
    plugs.call::<_, ()>("plug5", "hello_from_c", (10i32, 20i32))?;

    Ok(())
}

mod my_core {

    use wasmtime::Caller;
    use wlug::PlugContext;

    use crate::State;

    // `wasmtime` passes the correct `Caller` automatically when calling the function
    // You can omit the `Caller` arguement if you don't use it for any state management or memory access
    // `Caller` must be declared as the first arguement if you are going to use the state in this function
    // `PlugContext` struct is a tuple struct contains the id of the current plugin and your state, you can unpack
    // it and use that id according to your needs
    pub fn print(mut c: Caller<'_, PlugContext<State>>, a: i32) {
        // If we wanted to use strings or any other pointer from wasm memory, we could access the memory like this:
        // let memory = c
        //     .get_export("memory")
        //     .expect("Couldn't find 'memory' export")
        //     .into_memory()
        //     .unwrap();
        let PlugContext(id, state) = c.data_mut();
        println!(
            "[core::print]: plug{}: {a}; print_count: {}",
            *id + 1,
            state.print_count
        );
        state.print_count += 1;
    }

    pub fn print2(mut c: Caller<'_, PlugContext<State>>, x: i32, y: i32) {
        let PlugContext(id, state) = c.data_mut();
        println!(
            "[core::print2]: plug{}: {x},{y}; print2_count: {}",
            *id + 1,
            state.print2_count
        );
        state.print2_count += 1;
    }
}
