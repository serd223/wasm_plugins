use std::sync::{Arc, RwLock};

use embed::Plugs;
use wasmtime::*;

mod my_core {

    use std::sync::{Arc, RwLock};

    use crate::State;

    pub fn print(a: i32, state: &Arc<RwLock<State>>) {
        println!(
            "[core::print]: {a}; print_count: {}",
            state.read().unwrap().print_count
        );
        let mut state = state.write().unwrap();
        state.print_count += 1;
    }

    pub fn print2(x: i32, y: i32, state: &Arc<RwLock<State>>) {
        println!(
            "[core::print2]: {x},{y}; print2_count: {}",
            state.read().unwrap().print2_count
        );
        let mut state = state.write().unwrap();
        state.print2_count += 1;
    }
}

#[derive(Default)]
struct State {
    print_count: i32,
    print2_count: i32,
}

fn main() -> wasmtime::Result<()> {
    let engine = Engine::default();
    let mut plugs = Plugs::new(&engine);

    // Due to `wasmtime`s requirements regarding shared memory, we wrap our state in an `Arc` and a `RwLock`
    // `RwLock` isn't necessary if state isn't going to be mutated.
    let state = Arc::new(RwLock::new(State::default()));
    {
        let print_state = Arc::clone(&state);
        plugs.add_host_fn("print".to_string(), move |a: i32| {
            my_core::print(a, &print_state)
        });
        let print2_state = Arc::clone(&state);
        plugs.add_host_fn("print2".to_string(), move |x: i32, y: i32| {
            my_core::print2(x, y, &print2_state)
        });
    }

    // Load order is important and circular dependencies are disallowed
    plugs.add("../plug1.wasm", &engine)?;
    plugs.add("../plug2.wasm", &engine)?;
    plugs.add("../plug3.wasm", &engine)?;
    plugs.add("../plug4.wasm", &engine)?;
    plugs.add("../plug5.wasm", &engine)?;

    println!("[INFO]: Starting to link...");
    plugs.link()?;
    println!("\n[INFO]: Linking is complete.\n");

    println!("\n[INFO]: calling plug1.plug1 with args: 10");
    plugs.call::<_, ()>("plug1", "plug1", 10i32)?;

    println!("\n[INFO]: calling plug2.plug2 with args: 10");
    plugs.call::<_, ()>("plug2", "plug2", 10i32)?;

    println!("\n[INFO]: calling plug3.plug3 with args: 10");
    plugs.call::<_, ()>("plug3", "plug3", 10i32)?;

    println!("\n[INFO]: calling plug4.plug4 with args: 10");
    plugs.call::<_, ()>("plug4", "plug4", 10i32)?;

    println!("\n[INFO]: calling plug5.hello_from_c with args: (10, 20)");
    plugs.call::<_, ()>("plug5", "hello_from_c", (10i32, 20i32))?;

    Ok(())
}
