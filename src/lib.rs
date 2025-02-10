use std::{collections::HashMap, path::Path};

use wasmtime::{
    Engine, Extern, Func, Instance, IntoFunc, Linker, Module, Store, TypedFunc, WasmParams,
    WasmResults,
};

// Re-export wasmtime
pub use wasmtime;

pub const DEFAULT_DEPS_EXPORT: &str = "__deps";
pub const DEFAULT_INIT_EXPORT: &str = "__init";
pub const DEFAULT_RESET_EXPORT: &str = "__reset";
pub const DEFAULT_NAME_EXPORT: &str = "__name";

pub type PlugId = usize;

pub struct PlugContext<T>(pub PlugId, pub T);

pub struct Plug<T> {
    pub id: PlugId,
    pub module: Module,
    pub linker: Linker<PlugContext<T>>,
    pub instance: Option<Instance>,
    pub deps: Vec<String>,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
}

pub struct PlugMetadata {
    pub name: String,
    pub deps: Vec<String>,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
}

pub struct PlugsResetOptions<T> {
    pub plugs: bool,
    pub state: Option<T>,
    pub host_fns: bool,
}

impl<T> PlugsResetOptions<T> {
    /// If you want to reset the state, pass a Some(..) variant with the new value of the state, otherwise pass None
    pub fn new(plugs: bool, state: Option<T>, host_fns: bool) -> Self {
        Self {
            plugs,
            state,
            host_fns,
        }
    }
}

/// The main entry point of `wlug`, you can create a `Plugs` instance with `Plugs::new`
pub struct Plugs<'a, T> {
    pub store: Store<PlugContext<T>>,
    items: HashMap<String, Plug<T>>,
    order: Vec<String>,
    host_fns: Vec<(String, Extern)>,
    name_export: &'a str,
    deps_export: &'a str,
    init_export: &'a str,
    reset_export: &'a str,
}

impl<'a, T> Plugs<'a, T> {
    /// Create a new `Plugs` with a `wasmtime::Engine` and state
    pub fn new(engine: &Engine, state: T) -> Self {
        Self {
            store: Store::new(engine, PlugContext(0, state)),
            items: HashMap::new(),
            order: Vec::new(),
            host_fns: Vec::new(),
            name_export: DEFAULT_NAME_EXPORT,
            deps_export: DEFAULT_DEPS_EXPORT,
            init_export: DEFAULT_INIT_EXPORT,
            reset_export: DEFAULT_RESET_EXPORT,
        }
    }

    /// Change `name_export`
    pub fn with_name(self, name_export: &'a str) -> Self {
        Self {
            name_export,
            ..self
        }
    }

    /// Change `deps_export`
    pub fn with_deps(self, deps_export: &'a str) -> Self {
        Self {
            deps_export,
            ..self
        }
    }

    /// Change `init_export`
    pub fn with_init(self, init_export: &'a str) -> Self {
        Self {
            init_export,
            ..self
        }
    }

    /// Change `reset_export`
    pub fn with_reset(self, reset_export: &'a str) -> Self {
        Self {
            reset_export,
            ..self
        }
    }

    /// Returns a reference to the HashMap that contains plugin names and plugins
    pub fn items(&self) -> &HashMap<String, Plug<T>> {
        &self.items
    }

    /// Returns a reference to the list of plugin names in their load order
    pub fn order(&self) -> &Vec<String> {
        &self.order
    }

    /// Returns a reference to the list of functions supplied by the host
    pub fn host_fns(&self) -> &Vec<(String, Extern)> {
        &self.host_fns
    }

    /// Adds a new host function, function parameters and results are passed through the generic types
    pub fn add_host_fn<Params, Results>(
        &mut self,
        name: String,
        func: impl IntoFunc<PlugContext<T>, Params, Results>,
    ) {
        let func = Func::wrap(&mut self.store, func);
        let func = Into::<Extern>::into(func);
        self.host_fns.push((name, func));
    }

    /// Define host functions in the provided linker
    pub fn link_host(&mut self, linker: &mut Linker<PlugContext<T>>) -> wasmtime::Result<()> {
        for (name, func) in self.host_fns.iter() {
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
        id: PlugId,
    ) -> wasmtime::Result<PlugMetadata> {
        let imports = module
            .imports()
            .into_iter()
            .filter_map(|imp| {
                let is_host_fn = self.host_fns.iter().any(|(name, _)| name.eq(imp.name()));
                if !is_host_fn {
                    Some(imp.name().to_string())
                } else {
                    None
                }
            })
            .collect();
        let exports = module.exports().map(|e| e.name().to_string()).collect();

        let mut linker = Linker::new(engine);
        linker.define_unknown_imports_as_traps(&module)?;

        let instance = linker.instantiate(&mut self.store, &module)?;

        let memory = if let Some(m) = instance.get_export(&mut self.store, "memory") {
            if let Some(m) = m.into_memory() {
                m
            } else {
                return Err(wasmtime::Error::msg(format!(
                    "'memory' export isn't a WASM memory in plugin with id: {id}"
                )));
            }
        } else {
            return Err(wasmtime::Error::msg(format!(
                "Couldn't find 'memory' export in plugin with id: {id}"
            )));
        };

        // Extract dependencies (optional)
        let mut deps = Vec::new();
        if let Ok(deps_fn) = instance.get_typed_func::<(), u32>(&mut self.store, self.deps_export) {
            let mut deps_ptr = deps_fn.call(&mut self.store, ())? as usize;
            deps.push(String::new());
            let memory = memory.data(&mut self.store);
            while memory[deps_ptr] != 0 {
                let c = memory[deps_ptr] as char;
                if c == ';' {
                    deps.push(String::new());
                } else {
                    deps.last_mut().unwrap().push(c)
                }
                deps_ptr += 1;
            }
        }

        let mut name = String::new();
        match instance.get_typed_func::<(), u32>(&mut self.store, self.name_export) {
            Ok(name_fn) => {
                let mut name_ptr = name_fn.call(&mut self.store, ())? as usize;
                let memory = memory.data(&mut self.store);
                while memory[name_ptr] != 0 {
                    name.push(memory[name_ptr] as char);
                    name_ptr += 1;
                }
            }
            Err(_) => {
                return Err(wasmtime::Error::msg(format!(
                    "Couldn't find {} in module {module:?}",
                    self.name_export
                )))
            }
        }

        Ok(PlugMetadata {
            name,
            deps,
            exports,
            imports,
        })
    }

    /// Load wasm module and add it to the list of plugins. Will throw an error if the plugin name already exists.
    /// Will only link with host functions.
    /// Returns the id of the loaded plugin if load was successful
    pub fn load_module(&mut self, module: Module, engine: &Engine) -> wasmtime::Result<PlugId> {
        let id = self.order.len();
        let metadata = self.extract_metadata(engine, &module, id)?;

        let mut linker = Linker::new(engine);

        // Link host functions
        self.link_host(&mut linker)?;

        if self.items.contains_key(&metadata.name) {
            return Err(wasmtime::Error::msg(format!(
                "Plugin with name `{}` already exists",
                metadata.name
            )));
        }

        self.items.insert(
            metadata.name.clone(),
            Plug {
                id,
                module,
                linker,
                instance: None,
                deps: metadata.deps,
                exports: metadata.exports,
                imports: metadata.imports,
            },
        );
        self.order.push(metadata.name);

        Ok(id)
    }

    /// Load plugin from the provided binary and return is id (see `load_module`)
    pub fn load_binary(
        &mut self,
        bin: impl AsRef<[u8]>,
        engine: &Engine,
    ) -> wasmtime::Result<PlugId> {
        let module = Module::from_binary(engine, bin.as_ref())?;

        self.load_module(module, engine)
    }

    /// Load plugin from the file system and return its id (see `load_module`)
    pub fn load(
        &mut self,
        file_path: impl AsRef<Path>,
        engine: &Engine,
    ) -> wasmtime::Result<PlugId> {
        let module = Module::from_file(engine, file_path)?;

        self.load_module(module, engine)
    }

    /// Link all plugins, load order is important (TODO: auto sorting)
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
            let p = self.items.get(name.as_str()).unwrap();
            let deps = p.deps.clone();
            let mut imports = p.imports.clone();
            let mut to_import = Vec::new();

            // #[cfg(debug_assertions)]
            // println!("\n[Plugs::link]: '{name}' has {deps:?} as dependencies");

            if imports.len() > 0 {
                for dep_name in deps.iter() {
                    if let Some(p_dep) = self.items.get(dep_name) {
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

                                    // #[cfg(debug_assertions)]
                                    // println!("[Plugs::link]: Will define '{imp}' from '{dep_name}' in '{name}'");

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

    /// Reset `self` by clearing all plugins and calling their (optional) reset exports but doesn't reset the state inside `self.store`
    pub fn reset(&mut self) -> wasmtime::Result<()> {
        // order isn't important since this will be called after instantiation anyway
        for (_, p) in self.items.iter_mut() {
            if let Some(inst) = &p.instance {
                if let Ok(reset_fn) =
                    inst.get_typed_func::<(), ()>(&mut self.store, self.reset_export)
                {
                    self.store.data_mut().0 = p.id;
                    reset_fn.call(&mut self.store, ())?;
                }
            }
        }
        self.items.clear();
        self.order.clear();
        Ok(())
    }

    /// Reset `self` according to the given options
    pub fn reset_with_options(&mut self, options: PlugsResetOptions<T>) -> wasmtime::Result<()> {
        if options.plugs {
            self.reset()?;
        }
        if let Some(new_state) = options.state {
            *self.store.data_mut() = PlugContext(0, new_state);
        }
        if options.host_fns {
            self.host_fns.clear();
        }

        Ok(())
    }

    /// Return a '&' reference to the user defined state
    pub fn state(&self) -> &T {
        &self.store.data().1
    }

    /// Return a mutable reference to the user defined state
    pub fn state_mut(&mut self) -> &mut T {
        &mut self.store.data_mut().1
    }

    /// Call the init functions of all plugins. This method looks for an export matches `self.init_export`
    /// As an init export is optional in plugins, this method will just skip plugins without an init export.
    pub fn init(&mut self) -> wasmtime::Result<()> {
        let names = self.order.clone();

        for name in names {
            if let Ok((id, init_fn)) = self.get_func_with_id::<(), ()>(&name, self.init_export) {
                self.set_current_id(id);
                init_fn.call(&mut self.store, ())?;
            }
        }

        Ok(())
    }

    /// Convenience function for calling function in a plugin and setting the plugin's id as the current
    pub fn call<P: WasmParams, R: WasmResults>(
        &mut self,
        plug: &str,
        func: &str,
        params: P,
    ) -> wasmtime::Result<R> {
        let (id, f) = self.get_func_with_id(plug, func)?;
        self.set_current_id(id);
        f.call(&mut self.store, params)
    }

    /// Must be set before calling any function
    pub fn set_current_id(&mut self, plugin_id: PlugId) {
        self.store.data_mut().0 = plugin_id;
    }

    /// Look up a function in the specified plugin and return the id of the plugin and the function
    pub fn get_func_with_id<P: WasmParams, R: WasmResults>(
        &mut self,
        plug: &str,
        func: &str,
    ) -> wasmtime::Result<(PlugId, TypedFunc<P, R>)> {
        if let Some(p) = self.items.get(plug) {
            if let Some(inst) = &p.instance {
                inst.get_typed_func::<P, R>(&mut self.store, func)
                    .map(|f| (p.id, f))
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

    /// Get id of plugin by name
    pub fn get_id(&self, name: &str) -> Option<PlugId> {
        if let Some(p) = self.items.get(name) {
            return Some(p.id);
        }
        None
    }

    /// Get name of plugin by id
    pub fn get_name(&self, id: PlugId) -> Option<&String> {
        self.order.get(id)
    }

    /// Get reference to plugin by name
    pub fn get_plug(&self, name: &str) -> Option<&Plug<T>> {
        self.items.get(name)
    }

    /// Get mutable reference to plugin by name
    pub fn get_plug_mut(&mut self, name: &str) -> Option<&mut Plug<T>> {
        self.items.get_mut(name)
    }

    /// Get reference to plugin by id
    pub fn get_plug_id(&self, id: PlugId) -> Option<&Plug<T>> {
        if let Some(name) = self.order.get(id) {
            self.items.get(name)
        } else {
            None
        }
    }

    /// Get mutable reference to plugin by id
    pub fn get_plug_id_mut(&mut self, id: PlugId) -> Option<&mut Plug<T>> {
        if let Some(name) = self.order.get(id) {
            self.items.get_mut(name)
        } else {
            None
        }
    }
}
