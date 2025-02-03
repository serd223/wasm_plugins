use embed::Plugs;
use wasmtime::*;

mod my_core {
    use embed::PlugsLinker;

    pub fn link(mut linker: PlugsLinker) -> wasmtime::Result<()> {
        linker.define_fn("print", print)?;
        linker.define_fn("print2", print2)?;
        Ok(())
    }

    fn print(a: i32) {
        println!("[core::print]: {a}");
    }

    fn print2(x: i32, y: i32) {
        println!("[core::print2]: {x},{y}");
    }
}

fn main() -> wasmtime::Result<()> {
    let engine = Engine::default();
    let mut plugs = Plugs::new(&engine, Some(my_core::link));

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
