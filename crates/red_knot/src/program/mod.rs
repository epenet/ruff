use std::panic::{AssertUnwindSafe, RefUnwindSafe};
use std::sync::Arc;

use salsa::{Cancelled, Database};

use red_knot_module_resolver::{Db as ResolverDb, Jar as ResolverJar};
use red_knot_python_semantic::{Db as SemanticDb, Jar as SemanticJar};
use ruff_db::file_system::{FileSystem, FileSystemPathBuf};
use ruff_db::vfs::{Vfs, VfsFile, VfsPath};
use ruff_db::{Db as SourceDb, Jar as SourceJar, Upcast};

use crate::db::{Db, Jar};
use crate::Workspace;

mod check;

#[salsa::db(SourceJar, ResolverJar, SemanticJar, Jar)]
pub struct Program {
    storage: salsa::Storage<Program>,
    vfs: Vfs,
    fs: Arc<dyn FileSystem + Send + Sync + RefUnwindSafe>,
    workspace: Workspace,
}

impl Program {
    pub fn new<Fs>(workspace: Workspace, file_system: Fs) -> Self
    where
        Fs: FileSystem + 'static + Send + Sync + RefUnwindSafe,
    {
        Self {
            storage: salsa::Storage::default(),
            vfs: Vfs::default(),
            fs: Arc::new(file_system),
            workspace,
        }
    }

    pub fn apply_changes<I>(&mut self, changes: I)
    where
        I: IntoIterator<Item = FileWatcherChange>,
    {
        for change in changes {
            VfsFile::touch_path(self, &VfsPath::file_system(change.path));
        }
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspace
    }

    fn with_db<F, T>(&self, f: F) -> Result<T, Cancelled>
    where
        F: FnOnce(&Program) -> T + std::panic::UnwindSafe,
    {
        // (UN?)SAFETY: I don't think this is a 100% safe, lol. But it's what everyone else does.
        // The salsa storage is guaranteed to be unwind safe (or at least, has been designed to be unwind
        // safe for Salsa-exceptions). But this is not strictly guaranteed for any user code in `Program`.
        // For example, multi-threaded inside `Program` that calls a query could panic and unwind, before
        // it can complete and transitions into a valid state.
        // To me, this seems like a design flaw in salsa, see https://salsa.zulipchat.com/#narrow/stream/145099-general/topic/How.20to.20use.20.60Cancelled.3A.3Acatch.60
        let db = &AssertUnwindSafe(self);
        Cancelled::catch(|| f(db))
    }
}

impl Upcast<dyn SemanticDb> for Program {
    fn upcast(&self) -> &(dyn SemanticDb + 'static) {
        self
    }
}

impl Upcast<dyn SourceDb> for Program {
    fn upcast(&self) -> &(dyn SourceDb + 'static) {
        self
    }
}

impl Upcast<dyn ResolverDb> for Program {
    fn upcast(&self) -> &(dyn ResolverDb + 'static) {
        self
    }
}

impl ResolverDb for Program {}

impl SemanticDb for Program {}

impl SourceDb for Program {
    fn file_system(&self) -> &dyn FileSystem {
        &*self.fs
    }

    fn vfs(&self) -> &Vfs {
        &self.vfs
    }
}

impl Database for Program {}

impl Db for Program {}

impl salsa::ParallelDatabase for Program {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Self {
            storage: self.storage.snapshot(),
            vfs: self.vfs.snapshot(),
            fs: self.fs.clone(),
            workspace: self.workspace.clone(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct FileWatcherChange {
    path: FileSystemPathBuf,
    #[allow(unused)]
    kind: FileChangeKind,
}

impl FileWatcherChange {
    pub fn new(path: FileSystemPathBuf, kind: FileChangeKind) -> Self {
        Self { path, kind }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FileChangeKind {
    Created,
    Modified,
    Deleted,
}
