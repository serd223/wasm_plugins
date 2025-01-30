use std::{collections::HashMap, path::Path};

use wasmtime::{Engine, Instance, Linker, Module, Store, UnknownImportError};

mod plugs_core {
    pub fn link_core(linker: &mut wasmtime::Linker<()>) -> wasmtime::Result<()> {
        linker.func_wrap("env", "get_state", || 12)?;
        linker.func_wrap("env", "draw", |x: i32, y: i32| {
            draw(x as usize, y as usize);
        })?;
        linker.func_wrap("env", "print", print)?;
        Ok(())
    }
    pub fn draw(x: usize, y: usize) {
        println!("[plugs_core::draw]: Drawing at {x},{y}");
    }
    pub fn print(a: i32) {
        println!("[plugs_core::print]: {a}");
    }
}

pub struct Plug {
    module: Module,
    linker: Linker<()>,
    pub instance: Instance,
}

#[derive(Default)]
pub struct Plugs<'a> {
    pub store: Store<()>,
    pub items: HashMap<&'a str, Plug>,
    order: Vec<String>,
    deps: HashMap<String, Vec<String>>,
}

impl<'a> Plugs<'a> {
    pub fn new(engine: &Engine) -> Self {
        Self {
            store: Store::new(engine, ()),
            ..Default::default()
        }
    }
    /// Add plug (without linking except the core library)
    pub fn add(&mut self, file_path: &'a str, engine: &Engine) -> wasmtime::Result<()> {
        let fp = Path::new(file_path);
        let ext = fp.extension().unwrap();
        let ext_len = ext.len();
        let name = fp.file_name().unwrap().to_str().unwrap();
        let len = name.len();
        let name = &name[..len - ext_len - 1];
        let module = Module::from_file(engine, file_path)?;
        let mut linker = Linker::new(engine);
        linker.allow_shadowing(true);
        linker.define_unknown_imports_as_default_values(&module)?;

        // Link core library
        plugs_core::link_core(&mut linker)?;

        let instance = match linker.instantiate(&mut self.store, &module) {
            Ok(i) => i,
            Err(e) => {
                let e: UnknownImportError = e.downcast().unwrap();
                panic!("Error: {e:?}");
            }
        };

        // TODO: The plugin name could also be extracted in a similar way instead of
        // relying on the file name. The current file name approach makes the system simpler
        // but I think I will switch to a `name` export in the future.
        // Extract dependencies (optional)
        let deps = {
            let mut res = Vec::new();

            if let Ok(deps_fn) = instance.get_typed_func::<(), u32>(&mut self.store, "deps") {
                let mut deps_ptr = deps_fn.call(&mut self.store, ())?;
                let memory = instance
                    .get_memory(&mut self.store, "memory")
                    .expect("No 'memory' export");
                let mut deps_buf = vec![0u8];
                res.push(String::new());
                memory
                    .read(&mut self.store, deps_ptr as usize, &mut deps_buf)
                    .unwrap();
                while deps_buf[0] != 0 {
                    let c = deps_buf[0] as char;
                    if c == ';' {
                        res.push(String::new());
                    } else {
                        res.last_mut().unwrap().push(c);
                    }
                    deps_ptr += 1;
                    memory
                        .read(&mut self.store, deps_ptr as usize, &mut deps_buf)
                        .unwrap();
                }
            }
            res
        };
        self.deps.insert(String::from(name), deps);
        self.items.insert(
            name,
            Plug {
                module,
                linker,
                instance,
            },
        );
        self.order.push(name.to_string());

        Ok(())
    }

    /// Link all plugs, load order is important (TODO: auto sorting)
    /// and circular dependencies are disallowed (won't change, TODO: report as error)
    pub fn link(&mut self) -> wasmtime::Result<()> {
        // TODO: perhaps sort the plugins before linking them so that all plugins are guaranteed to be loaded after their dependencies
        // this could also be a chance for us to detect circular dependencies and throw an error in that case since they are disallowed
        //
        // Circular dependencies are disallowed because we can't easily detect which _symbol_ depends on which, we only know which plugin
        // depends on which symbols and that isn't really enough to properly resolve all cases. If we were to just use that info, there
        // could be some edge case where the linker doesn't properly link everything especially if the dependency graph is very
        // convoluted and the circular dependency is deep within the dependency tree.
        for name in self.order.iter() {
            let deps = self.deps.get(name).unwrap();
            println!("\n[Plugs::link]: {name} has {deps:?} as dependencies");
            let p = std::ptr::from_mut(self.items.get_mut(name.as_str()).unwrap());
            for dep in deps.iter() {
                if let Some(p_dep) = self.items.get_mut(dep.as_str()) {
                    let exports = p_dep
                        .instance
                        .exports(&mut self.store)
                        .map(|e| (e.name().to_string(), e.into_extern()))
                        .collect::<Vec<_>>();
                    for (key, export) in exports {
                        if !["memory", "__data_end", "__heap_base", "deps", "name"] // Reserved exports
                            .contains(&key.as_str())
                        {
                            println!("[Plugs::link]: Defining '{key}' from '{dep}' in '{name}'");
                            // Technically unsafe but realistically completely safe
                            unsafe {
                                (*p).linker
                                    .define(&mut self.store, "env", key.as_str(), export)?
                            };
                        }
                    }
                } else {
                    panic!("[Plugs::link]: {dep} is not a valid import.");
                }
            }
            if deps.len() > 0 {
                let p = self.items.get_mut(name.as_str()).unwrap();
                p.instance = p.linker.instantiate(&mut self.store, &p.module)?;
            }
        }
        Ok(())
    }
}
