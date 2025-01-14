use std::fmt::Formatter;
use std::sync::Arc;

use ruff_db::vfs::VfsFile;

use crate::db::Db;
use crate::module_name::ModuleName;
use crate::path::{ModuleResolutionPathBuf, ModuleResolutionPathRef};

/// Representation of a Python module.
#[derive(Clone, PartialEq, Eq)]
pub struct Module {
    inner: Arc<ModuleInner>,
}

impl Module {
    pub(crate) fn new(
        name: ModuleName,
        kind: ModuleKind,
        search_path: Arc<ModuleResolutionPathBuf>,
        file: VfsFile,
    ) -> Self {
        Self {
            inner: Arc::new(ModuleInner {
                name,
                kind,
                search_path,
                file,
            }),
        }
    }

    /// The absolute name of the module (e.g. `foo.bar`)
    pub fn name(&self) -> &ModuleName {
        &self.inner.name
    }

    /// The file to the source code that defines this module
    pub fn file(&self) -> VfsFile {
        self.inner.file
    }

    /// The search path from which the module was resolved.
    pub(crate) fn search_path(&self) -> ModuleResolutionPathRef {
        ModuleResolutionPathRef::from(&*self.inner.search_path)
    }

    /// Determine whether this module is a single-file module or a package
    pub fn kind(&self) -> ModuleKind {
        self.inner.kind
    }
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("name", &self.name())
            .field("kind", &self.kind())
            .field("file", &self.file())
            .field("search_path", &self.search_path())
            .finish()
    }
}

impl salsa::DebugWithDb<dyn Db> for Module {
    fn fmt(&self, f: &mut Formatter<'_>, db: &dyn Db) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("name", &self.name())
            .field("kind", &self.kind())
            .field("file", &self.file().debug(db.upcast()))
            .field("search_path", &self.search_path())
            .finish()
    }
}

#[derive(PartialEq, Eq)]
struct ModuleInner {
    name: ModuleName,
    kind: ModuleKind,
    search_path: Arc<ModuleResolutionPathBuf>,
    file: VfsFile,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ModuleKind {
    /// A single-file module (e.g. `foo.py` or `foo.pyi`)
    Module,

    /// A python package (`foo/__init__.py` or `foo/__init__.pyi`)
    Package,
}
