use std::{collections::HashMap, path::Path};

use wasmtime::{
    Engine, Extern, Func, Instance, IntoFunc, Linker, Module, Store, TypedFunc, UnknownImportError,
    Val, ValType, WasmParams, WasmResults,
};

pub struct Plug {
    pub module: Module,
    pub linker: Linker<()>,
    pub instance: Option<Instance>,
    pub deps: Vec<String>,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
}

pub struct PlugMetadata {
    pub deps: Vec<String>,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
}

pub struct PlugsHostFns {
    pub fns: Vec<(String, Extern)>,
}

pub struct Plugs {
    pub store: Store<()>,
    pub items: HashMap<String, Plug>,
    pub order: Vec<String>,
    pub host_fns: PlugsHostFns,
}

impl Plugs {
    /// Create a new `Plugs` with a `wasmtime::Engine`, optional state and an optional core linking function if you want to have core functions for your plugins
    /// You will usually need to wrap your state in a `Mutex` or a `Rwlock` if you want to mutate it as `wasmtime` has certain requirements regarding shared memory
    /// The state is internally stored in an `Arc` (which is why the core_linker accepts &Option<Arc<T>>) so you don't have to wrap your type in an `Arc` yourself
    pub fn new(engine: &Engine) -> Self {
        Self {
            store: Store::new(engine, ()),
            items: HashMap::new(),
            order: Vec::new(),
            host_fns: PlugsHostFns { fns: Vec::new() },
        }
    }

    pub fn add_host_fn<Params, Results>(
        &mut self,
        name: String,
        func: impl IntoFunc<(), Params, Results>,
    ) {
        let func = Into::<Extern>::into(Func::wrap(&mut self.store, func));
        self.host_fns.fns.push((name, func));
    }

    pub fn link_host(&mut self, linker: &mut Linker<()>) -> wasmtime::Result<()> {
        for (name, func) in self.host_fns.fns.iter() {
            linker.define(&mut self.store, "env", name, func.clone())?;
        }
        Ok(())
    }

    /// Extract metadata from the specified module by instantiating a temporary instance and running the
    /// necessary reserved functions (such as `deps`) for metadata extraction.
    pub fn extract_metadata(
        &mut self,
        engine: &Engine,
        module: &Module,
    ) -> wasmtime::Result<PlugMetadata> {
        let mut linker = Linker::new(engine);

        let mut imports = Vec::new();
        let instance = loop {
            match linker.instantiate(&mut self.store, &module) {
                Ok(inst) => break inst,
                Err(e) => {
                    let e: UnknownImportError = e.downcast()?;
                    let ftype = e.ty().func().unwrap().clone();
                    let result_types = ftype.results().collect::<Vec<_>>();
                    linker.func_new("env", e.name(), ftype, move |_, _, results| {
                        for (i, res_type) in result_types.iter().enumerate() {
                            results[i] = match res_type {
                                ValType::I32 => Val::I32(0),
                                ValType::I64 => Val::I64(0),
                                ValType::F32 => Val::F32(0f32.to_bits()),
                                ValType::F64 => Val::F64(0f64.to_bits()),
                                ValType::V128 => Val::V128(0u128.into()),
                                ValType::Ref(r) => Val::null_ref(r.heap_type()),
                            };
                        }

                        Ok(())
                    })?;
                    let imp = e.name().to_string();
                    let is_host_fn = self.host_fns.fns.iter().any(|(n, _)| imp.eq(n));
                    if !is_host_fn {
                        imports.push(e.name().to_string());
                    }
                    continue;
                }
            }
        };

        // TODO: The plugin name could also be extracted in a similar way instead of
        // relying on the file name. The current file name approach makes the system simpler
        // but I think I will switch to a `name` export in the future.

        // Extract dependencies (optional)
        let mut deps = Vec::new();
        if let Ok(deps_fn) = instance.get_typed_func::<(), u32>(&mut self.store, "deps") {
            let mut deps_ptr = deps_fn.call(&mut self.store, ())?;
            let memory = {
                if let Some(m) = instance.get_memory(&mut self.store, "memory") {
                    m
                } else {
                    return Err(wasmtime::Error::msg("Couldn't find 'memory' export"));
                }
            };
            let mut deps_buf = vec![0u8];
            deps.push(String::new());
            memory.read(&mut self.store, deps_ptr as usize, &mut deps_buf)?;
            while deps_buf[0] != 0 {
                let c = deps_buf[0] as char;
                if c == ';' {
                    deps.push(String::new());
                } else {
                    deps.last_mut().unwrap().push(c);
                }
                deps_ptr += 1;
                memory.read(&mut self.store, deps_ptr as usize, &mut deps_buf)?;
            }
        }
        let exports = module.exports().map(|e| e.name().to_string()).collect();
        Ok(PlugMetadata {
            deps,
            exports,
            imports,
        })
    }

    /// Add plug (without linking except the core library)
    pub fn add(&mut self, file_path: &str, engine: &Engine) -> wasmtime::Result<()> {
        let fp = Path::new(file_path);
        let ext = fp.extension().unwrap();
        let ext_len = ext.len();
        let name = fp.file_name().unwrap().to_str().unwrap();
        let len = name.len();
        let name = &name[..len - ext_len - 1];
        let module = Module::from_file(engine, file_path)?;

        let metadata = self.extract_metadata(engine, &module)?;

        let mut linker = Linker::new(engine);

        // Link core library (optional)
        self.link_host(&mut linker)?;

        self.items.insert(
            name.to_string(),
            Plug {
                module,
                linker,
                instance: None,
                deps: metadata.deps,
                exports: metadata.exports,
                imports: metadata.imports,
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
            let p = self.items.get_mut(name.as_str()).unwrap();
            let deps = p.deps.clone();
            let mut imports = p.imports.clone();
            let mut to_import = Vec::new();

            #[cfg(debug_assertions)]
            println!("\n[Plugs::link]: '{name}' has {deps:?} as dependencies");

            if imports.len() > 0 {
                for dep_name in deps.iter() {
                    if let Some(p_dep) = self.items.get_mut(dep_name) {
                        imports = {
                            let mut res = Vec::new();
                            for imp in imports {
                                let exists = p_dep.exports.contains(&imp);
                                if exists {
                                    let inst = if let Some(inst) = &p_dep.instance {
                                        inst
                                    } else {
                                        return Err(wasmtime::Error::msg(format!("Dependency '{dep_name}' in plugin '{name}' hasn't been instantiated yet")));
                                    };

                                    let export = if let Some(e) =
                                        inst.get_export(&mut self.store, &imp)
                                    {
                                        e
                                    } else {
                                        return Err(wasmtime::Error::msg(format!("Dependency '{dep_name}' doesn't have export '{imp}' required by plugin '{name}'")));
                                    };

                                    #[cfg(debug_assertions)]
                                    println!("[Plugs::link]: Will define '{imp}' from '{dep_name}' in '{name}'");

                                    // plug.imports should never contain any reserved exports unless something went very wrong
                                    to_import.push((imp, export));
                                } else {
                                    res.push(imp);
                                }
                            }

                            res
                        };
                    } else {
                        return Err(wasmtime::Error::msg(format!(
                            "'{dep_name}' is not a valid dependency"
                        )));
                    }
                }
            }

            let p = self.items.get_mut(name.as_str()).unwrap();

            if imports.len() > 0 {
                return Err(wasmtime::Error::msg(format!(
                    "Plugin '{name}' has unresolved imports: {:?}",
                    imports
                )));
            }

            for (imp, export) in to_import {
                p.linker.define(&mut self.store, "env", &imp, export)?;
            }

            p.instance = Some(p.linker.instantiate(&mut self.store, &p.module)?);
        }
        Ok(())
    }

    /// Convenience function for getting and calling function in a plugin
    pub fn call<P: WasmParams, R: WasmResults>(
        &mut self,
        plug: &str,
        func: &str,
        params: P,
    ) -> wasmtime::Result<R> {
        let f = self.get_func(plug, func)?;
        f.call(&mut self.store, params)
    }

    /// Looks up a function in the specified plugin
    pub fn get_func<P: WasmParams, R: WasmResults>(
        &mut self,
        plug: &str,
        func: &str,
    ) -> Result<TypedFunc<P, R>, wasmtime::Error> {
        if let Some(p) = self.items.get_mut(plug) {
            if let Some(inst) = &p.instance {
                inst.get_typed_func::<P, R>(&mut self.store, func)
            } else {
                Err(wasmtime::Error::msg(format!(
                    "Plugin '{plug}' hasn't been instantiated yet"
                )))
            }
        } else {
            Err(wasmtime::Error::msg(format!(
                "Couldn't find function '{func}' in plugin '{plug}'"
            )))
        }
    }

    pub fn get_plug_mut(&mut self, name: &str) -> Option<&mut Plug> {
        self.items.get_mut(name)
    }

    pub fn get_plug(&self, name: &str) -> Option<&Plug> {
        self.items.get(name)
    }
}
