//! Boundary: reads, once, every file the probes judge. All I/O goes through
//! [`vitals_utils::fs`]; the result is a plain value.

use std::path::{Path, PathBuf};

use vitals_utils::markers::{AGENTS_MD, skill_files};
use vitals_utils::{Error, fs};

use crate::cascade::{ANCESTOR_FILES, classify, is_root_file};

/// How far a run looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Reach {
    /// The repository and the user scope: every parent directory up to the filesystem
    /// root, and the home directory.
    #[default]
    WithUserScope,
    /// The repository only, for reproducible CI runs.
    RepositoryOnly,
}

/// An instruction file: its content and, when it is a symbolic link, its target as written.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RulesFile {
    pub(crate) content: Option<String>,
    pub(crate) link: Option<PathBuf>,
}

/// An instruction file found by a walk, at `path`: relative to the root inside the
/// repository, absolute above it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) path: PathBuf,
    pub(crate) file: RulesFile,
}

/// What the probes see of a repository.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Snapshot {
    pub(crate) agents_md: Option<String>,
    pub(crate) claude_md: RulesFile,
    pub(crate) dot_claude_md: RulesFile,
    pub(crate) claude_local_md: bool,
    pub(crate) todo_md: bool,
    /// The first `kaizen` skill found, relative to the root.
    pub(crate) kaizen_skill: Option<PathBuf>,
    /// Every other instruction file in the repository, in walk order.
    pub(crate) nested: Vec<Entry>,
    /// `opencode.json` and `opencode.jsonc`, concatenated, when either exists.
    pub(crate) opencode_config: Option<String>,
    /// Instruction files in the parent directories; `None` when the run is repository-only.
    pub(crate) ancestors: Option<Vec<Entry>>,
    /// The home directory, when the run reaches the user scope.
    pub(crate) home: Option<PathBuf>,
}

pub(crate) fn read(root: &Path, reach: Reach) -> Result<Snapshot, Error> {
    fs::require_dir(root)?;
    let user = reach == Reach::WithUserScope;
    Ok(Snapshot {
        agents_md: fs::read_optional(&root.join(AGENTS_MD))?,
        claude_md: rules_file(&root.join("CLAUDE.md"))?,
        dot_claude_md: rules_file(&root.join(".claude/CLAUDE.md"))?,
        claude_local_md: exists(&root.join("CLAUDE.local.md"))?,
        todo_md: exists(&root.join("TODO.md"))?,
        kaizen_skill: kaizen_skill(root)?,
        nested: nested(root)?,
        opencode_config: opencode_config(root)?,
        ancestors: user.then(|| ancestors(root)).transpose()?,
        home: user.then(fs::home_dir).flatten(),
    })
}

fn rules_file(path: &Path) -> Result<RulesFile, Error> {
    Ok(RulesFile {
        content: fs::read_optional(path)?,
        link: fs::link_target(path)?,
    })
}

fn entry(path: PathBuf, on_disk: &Path) -> Result<Option<Entry>, Error> {
    let file = rules_file(on_disk)?;
    let present = file.content.is_some() || file.link.is_some();
    Ok(present.then_some(Entry { path, file }))
}

fn exists(path: &Path) -> Result<bool, Error> {
    fs::read_optional(path).map(|content| content.is_some())
}

fn kaizen_skill(root: &Path) -> Result<Option<PathBuf>, Error> {
    skill_files("kaizen")
        .map(|relative| {
            fs::read_optional(&root.join(&relative)).map(|found| found.map(|_| relative))
        })
        .find_map(Result::transpose)
        .transpose()
}

fn nested(root: &Path) -> Result<Vec<Entry>, Error> {
    fs::project_files(root)?
        .into_iter()
        .filter(|path| classify(path).is_some() && !is_root_file(path))
        .map(|path| {
            let on_disk = root.join(&path);
            entry(path, &on_disk)
        })
        .filter_map(Result::transpose)
        .collect()
}

fn opencode_config(root: &Path) -> Result<Option<String>, Error> {
    let parts = ["opencode.json", "opencode.jsonc"]
        .iter()
        .map(|name| fs::read_optional(&root.join(name)))
        .collect::<Result<Vec<_>, _>>()?;
    let found: Vec<String> = parts.into_iter().flatten().collect();
    Ok((!found.is_empty()).then(|| found.join("\n")))
}

fn ancestors(root: &Path) -> Result<Vec<Entry>, Error> {
    let root = fs::canonical(root)?;
    root.ancestors()
        .skip(1)
        .flat_map(|dir| ANCESTOR_FILES.iter().map(move |name| dir.join(name)))
        .map(|path| entry(path.clone(), &path))
        .filter_map(Result::transpose)
        .collect()
}
