use wasmtime::ExternType;

use crate::PlugId;

#[derive(Clone, Debug)]
pub struct PluginAlreadyExists {
    pub(crate) name: String,
}

impl std::fmt::Display for PluginAlreadyExists {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Plugin with name '{}' already exists", self.name)
    }
}

impl core::error::Error for PluginAlreadyExists {}

#[derive(Clone, Debug)]
pub enum UnknownPlugin {
    Id(PlugId),
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
    NotInstantiated {
        dep_name: String,
        plug_name: String,
    },
    ExportNotFound {
        dep_name: String,
        export_name: String,
        plug_name: String,
    },
    InvalidDependency(String),
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
            LinkError::InvalidDependency(dep_name) => write!(f, "'{dep_name}' is not a valid dependency"),
            LinkError::UnresolvedImports {
                plug_name,
                unresolved_imports,
            } => write!(f, "Plugin '{plug_name}' has unresolved imports: {unresolved_imports:?}",
),
        }
    }
}

impl core::error::Error for LinkError {}
