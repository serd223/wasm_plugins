use wasmtime::{ExternType, ValType};

use crate::PlugId;

#[derive(Clone, Debug)]
/// "Plugin with name '{name}' already exists"
pub struct PluginAlreadyExists {
    pub(crate) name: String,
}

impl PluginAlreadyExists {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Display for PluginAlreadyExists {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Plugin with name '{}' already exists", self.name)
    }
}

impl core::error::Error for PluginAlreadyExists {}

#[derive(Clone, Debug)]
pub enum UnknownPlugin {
    /// "Plugin with id '{id}' couldn't be found"
    Id(PlugId),

    /// "Plugin '{name}' couldn't be found"
    Name(String),
}

impl std::fmt::Display for UnknownPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnknownPlugin::Id(id) => {
                write!(f, "Plugin with id '{id}' couldn't be found")
            }
            UnknownPlugin::Name(name) => write!(f, "Plugin '{name}' couldn't be found"),
        }
    }
}

impl core::error::Error for UnknownPlugin {}

#[derive(Clone, Debug)]
pub enum ExportType {
    Memory,
    Func,
}

impl From<ExternType> for ExportType {
    fn from(value: ExternType) -> Self {
        match value {
            ExternType::Func(_) => ExportType::Func,
            ExternType::Global(_) => {
                panic!("wlug::ExportType can't be created from wasmtime::ExternType::Global")
            }
            ExternType::Table(_) => {
                panic!("wlug::ExportType can't be created from wasmtime::ExternType::Table")
            }
            ExternType::Memory(_) => ExportType::Memory,
        }
    }
}

#[derive(Debug)]
/// "Export '{export_name}' not found in plugin '{plug_name}'",
/// The `expected_ty` field stores the expected type of the export which was not found.
pub struct ExportNotFound {
    pub(crate) export_name: String,
    pub(crate) plug_name: String,
    pub(crate) expected_ty: ExportType,
}

impl ExportNotFound {
    pub fn export_name(&self) -> &str {
        &self.export_name
    }

    pub fn plug_name(&self) -> &str {
        &self.plug_name
    }

    pub fn expected_ty(&self) -> ExportType {
        self.expected_ty.clone()
    }
}

impl std::fmt::Display for ExportNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Export '{}' not found in plugin '{}'",
            self.export_name, self.plug_name
        )
    }
}

impl core::error::Error for ExportNotFound {}

#[derive(Clone, Debug)]
pub enum LinkError {
    /// "Dependency '{dep_name}' in plugin '{plug_name}' hasn't been instantiated yet"
    NotInstantiated { dep_name: String, plug_name: String },

    /// "Dependency '{dep_name}' doesn't have export '{export_name}' required by plugin '{plug_name}'"
    ExportNotFound {
        dep_name: String,
        export_name: String,
        plug_name: String,
    },

    /// "Dependency '{dep_name}' couldn't be found"
    DependencyNotFound(String),

    /// "Plugin '{plug_name}' has unresolved imports: {unresolved_imports:?}"
    UnresolvedImports {
        plug_name: String,
        unresolved_imports: Vec<String>,
    },
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkError::NotInstantiated {
                dep_name,
                plug_name,
            } => write!(
                f,
                "Dependency '{dep_name}' in plugin '{plug_name}' hasn't been instantiated yet"
            ),
            LinkError::ExportNotFound {
                dep_name,
                export_name,
                plug_name,
            } => write!(f, "Dependency '{dep_name}' doesn't have export '{export_name}' required by plugin '{plug_name}'"),
            LinkError::DependencyNotFound(dep_name) => write!(f, "Dependency '{dep_name}' couldn't be found"),
            LinkError::UnresolvedImports {
                plug_name,
                unresolved_imports,
            } => write!(f, "Plugin '{plug_name}' has unresolved imports: {unresolved_imports:?}",
),
        }
    }
}

impl core::error::Error for LinkError {}

/// Expected a plugin's function export's arguements to have a certain signature in a dynamic call but it had a different signature.
#[derive(Debug)]
pub struct DynamicDispatchError {
    pub(crate) func_name: String,
    pub(crate) plugin_name: String,
    pub(crate) expected_signature: Vec<ValType>,
    pub(crate) actual_signature: Vec<ValType>,
}

impl DynamicDispatchError {
    pub fn fun_name(&self) -> &String {
        &self.func_name
    }

    pub fn plugin_name(&self) -> &String {
        &self.plugin_name
    }

    pub fn expected_signature(&self) -> &Vec<ValType> {
        &self.expected_signature
    }

    pub fn actual_signature(&self) -> &Vec<ValType> {
        &self.actual_signature
    }
}

impl std::fmt::Display for DynamicDispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expected the arguements of function '{}' in plugin '{}' to have the following signature: {:?} but they had this signature: {:?}", self.func_name, self.plugin_name, self.expected_signature, self.actual_signature)
    }
}

impl core::error::Error for DynamicDispatchError {}
