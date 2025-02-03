use std::sync::RwLock;

use embed::Plugs;
use wasmtime::*;

mod my_core {
    use embed::PlugsLinker;
    use std::sync::{Arc, RwLock};

    use crate::AppState;

    // Due to `wasmtime`s requirements, we need to wrap our state in a `RwLock` to be able to mutate it
    type AppStateType = RwLock<AppState>;

    pub fn link(
        mut linker: PlugsLinker,
        state: &Option<Arc<AppStateType>>,
    ) -> wasmtime::Result<()> {
        let print_state = state.clone().expect("State is None");
        let print2_state = Arc::clone(&print_state);

        linker.define_fn("print", move |a: i32| print(a, &print_state))?;
        linker.define_fn("print2", move |x: i32, y: i32| print2(x, y, &print2_state))?;
        Ok(())
    }

    fn print(a: i32, state: &Arc<AppStateType>) {
        println!(
            "[core::print]: {a}; print_count: {}",
            state.read().unwrap().print_count
        );
        let mut state = state.write().unwrap();
        state.print_count += 1;
    }

    fn print2(x: i32, y: i32, state: &Arc<AppStateType>) {
        println!(
            "[core::print2]: {x},{y}; print2_count: {}",
            state.read().unwrap().print2_count
        );
        let mut state = state.write().unwrap();
        state.print2_count += 1;
    }
}

#[derive(Default)]
struct AppState {
    print_count: i32,
    print2_count: i32,
}

fn main() -> wasmtime::Result<()> {
    let engine = Engine::default();
    // States and core libraries are both optional
    let mut plugs = Plugs::new(
        &engine,
        Some(RwLock::new(AppState::default())),
        Some(my_core::link),
    );

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
