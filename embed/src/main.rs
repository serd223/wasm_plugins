use embed::Plugs;
use wasmtime::*;

fn main() -> wasmtime::Result<()> {
    let engine = Engine::default();
    let mut plugs = Plugs::new(&engine);

    // Load order is important and circular dependencies are disallowed
    plugs.add("../plug1.wasm", &engine)?;
    plugs.add("../plug2.wasm", &engine)?;
    plugs.add("../plug3.wasm", &engine)?;
    plugs.add("../plug4.wasm", &engine)?;
    plugs.add("../plug5.wasm", &engine)?;

    println!("[INFO]: Starting to link...");
    plugs.link()?;
    println!("\n[INFO]: Linking is complete.\n");

    if let Some(plug1) = plugs.items.get_mut("plug1") {
        let plug1_run = plug1
            .instance
            .get_typed_func::<i32, ()>(&mut plugs.store, "plug1")?;
        println!("[INFO]: calling plug1 with args: 10");
        plug1_run.call(&mut plugs.store, 10)?;
    }
    if let Some(plug2) = plugs.items.get_mut("plug2") {
        let plug2_run = plug2
            .instance
            .get_typed_func::<i32, ()>(&mut plugs.store, "plug2")?;
        println!("\n[INFO]: calling plug2 with args: 10");
        plug2_run.call(&mut plugs.store, 10)?;
    }
    if let Some(plug3) = plugs.items.get_mut("plug3") {
        let plug3_run = plug3
            .instance
            .get_typed_func::<i32, ()>(&mut plugs.store, "plug3")?;
        println!("\n[INFO]: calling plug3 with args: 10");
        plug3_run.call(&mut plugs.store, 10)?;
    }
    if let Some(plug4) = plugs.items.get_mut("plug4") {
        let plug4_run = plug4
            .instance
            .get_typed_func::<i32, ()>(&mut plugs.store, "plug4")?;
        println!("\n[INFO]: calling plug4 with args: 10");
        plug4_run.call(&mut plugs.store, 10)?;
    }
    if let Some(plug5) = plugs.items.get_mut("plug5") {
        let plug5_run = plug5
            .instance
            .get_typed_func::<(i32, i32), ()>(&mut plugs.store, "hello_from_c")?;
        println!("\n[INFO]: calling plug5 with args: (10, 20)");
        plug5_run.call(&mut plugs.store, (10, 20))?;
    }

    Ok(())
}
