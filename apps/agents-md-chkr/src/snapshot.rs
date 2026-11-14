//! Boundary: reads, once, every file the probes judge. All I/O goes through
//! [`vitals_utils::fs`]; the result is a plain value.

use std::path::{Path, PathBuf};

use vitals_utils::markers::{AGENTS_MD, skill_files};
use vitals_utils::{Error, fs};

/// A rules file that can shadow `AGENTS.md` in Claude Code: its content and, when it is a
/// symbolic link, its target as written.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Shadow {
    pub(crate) content: Option<String>,
    pub(crate) link: Option<PathBuf>,
}

/// What the probes see of a repository.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Snapshot {
    pub(crate) agents_md: Option<String>,
    pub(crate) claude_md: Shadow,
    pub(crate) dot_claude_md: Shadow,
    pub(crate) claude_local_md: bool,
    pub(crate) todo_md: bool,
    /// The first `kaizen` skill found, relative to the root.
    pub(crate) kaizen_skill: Option<PathBuf>,
}

pub(crate) fn read(root: &Path) -> Result<Snapshot, Error> {
    fs::require_dir(root)?;
    Ok(Snapshot {
        agents_md: fs::read_optional(&root.join(AGENTS_MD))?,
        claude_md: shadow(&root.join("CLAUDE.md"))?,
        dot_claude_md: shadow(&root.join(".claude/CLAUDE.md"))?,
        claude_local_md: exists(&root.join("CLAUDE.local.md"))?,
        todo_md: exists(&root.join("TODO.md"))?,
        kaizen_skill: kaizen_skill(root)?,
    })
}

fn shadow(path: &Path) -> Result<Shadow, Error> {
    Ok(Shadow {
        content: fs::read_optional(path)?,
        link: fs::link_target(path)?,
    })
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
