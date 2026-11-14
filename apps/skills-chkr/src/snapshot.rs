//! Boundary: reads, once, every file the probes judge. All I/O goes through
//! [`vitals_utils::fs`]; the result is a plain value.

use std::path::{Path, PathBuf};

use vitals_utils::markers::SKILL_ROOTS;
use vitals_utils::{Error, Reach, fs};

use crate::layout::{Place, SKILL_MD, folder_place, skill_place};
use crate::user::HOME_ROOTS;

/// One skill in the repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Skill {
    pub(crate) place: Place,
    /// The content of its `SKILL.md`.
    pub(crate) text: String,
    /// Every file in the skill directory, relative to it (symbolic-link folders followed).
    pub(crate) files: Vec<PathBuf>,
}

/// A skill root outside the repository and the skills it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutsideRoot {
    /// Absolute path of the root directory.
    pub(crate) path: PathBuf,
    /// Names of the skill directories in it, sorted.
    pub(crate) skills: Vec<String>,
}

/// What the user-scope probes see.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct User {
    pub(crate) home: Option<PathBuf>,
    /// Skill roots in the parent directories and the home directory, nearest first.
    pub(crate) roots: Vec<OutsideRoot>,
}

/// What the probes see of a repository.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Snapshot {
    /// Every skill, sorted by place.
    pub(crate) skills: Vec<Skill>,
    /// Every file in the repository, relative to the root, sorted.
    pub(crate) files: Vec<PathBuf>,
    /// `None` when the run is repository-only.
    pub(crate) user: Option<User>,
}

pub(crate) fn read(root: &Path, reach: Reach) -> Result<Snapshot, Error> {
    fs::require_dir(root)?;
    let mut files = fs::project_files(root)?;
    files.sort();
    Ok(Snapshot {
        skills: skills(root, &files)?,
        user: reach
            .includes_user_scope()
            .then(|| user(root))
            .transpose()?,
        files,
    })
}

fn skills(root: &Path, files: &[PathBuf]) -> Result<Vec<Skill>, Error> {
    let mut places = files
        .iter()
        .map(|path| place(root, path))
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>, _>>()?;
    places.sort();
    places.dedup();
    places
        .into_iter()
        .map(|place| skill(root, place))
        .filter_map(Result::transpose)
        .collect()
}

/// The skill that `path` makes: its `SKILL.md`, or a symbolic-link folder holding one.
fn place(root: &Path, path: &Path) -> Result<Option<Place>, Error> {
    if let Some(place) = skill_place(path) {
        return Ok(Some(place));
    }
    let Some(place) = folder_place(path) else {
        return Ok(None);
    };
    let on_disk = root.join(path);
    let linked = fs::link_target(&on_disk)?.is_some() && fs::exists(&on_disk.join(SKILL_MD))?;
    Ok(linked.then_some(place))
}

fn skill(root: &Path, place: Place) -> Result<Option<Skill>, Error> {
    let dir = root.join(place.dir());
    let Some(text) = fs::read_optional(&dir.join(SKILL_MD))? else {
        return Ok(None);
    };
    let mut files = fs::project_files(&dir)?;
    files.sort();
    Ok(Some(Skill { place, text, files }))
}

fn user(root: &Path) -> Result<User, Error> {
    // `$HOME` may be a symbolic link; the parent walk is resolved, so resolve it too.
    let home = fs::home_dir().map(|home| fs::canonical(&home).unwrap_or(home));
    let parents = fs::parent_dirs(root)?
        .into_iter()
        .flat_map(|dir| SKILL_ROOTS.iter().map(move |name| dir.join(name)));
    let personal = home
        .iter()
        .flat_map(|home| HOME_ROOTS.iter().map(move |name| home.join(name)));
    let candidates = parents.chain(personal).fold(Vec::new(), |mut seen, path| {
        if !seen.contains(&path) {
            seen.push(path);
        }
        seen
    });
    let roots = candidates
        .into_iter()
        .map(outside_root)
        .filter_map(Result::transpose)
        .collect::<Result<_, _>>()?;
    Ok(User { home, roots })
}

fn outside_root(path: PathBuf) -> Result<Option<OutsideRoot>, Error> {
    if !fs::exists(&path)? {
        return Ok(None);
    }
    let mut skills = fs::project_files(&path)?
        .into_iter()
        .filter_map(|file| skill_name(&path, &file).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    skills.sort();
    skills.dedup();
    Ok((!skills.is_empty()).then_some(OutsideRoot { path, skills }))
}

/// The skill name that `file` (relative to the root `dir`) makes: `<name>/SKILL.md`, or a
/// symbolic-link folder `<name>` holding one.
fn skill_name(dir: &Path, file: &Path) -> Result<Option<String>, Error> {
    let parts: Vec<&str> = file.iter().filter_map(|part| part.to_str()).collect();
    match parts.as_slice() {
        [name, SKILL_MD] => Ok(Some((*name).to_owned())),
        [name] => {
            let folder = dir.join(name);
            let linked = fs::link_target(&folder)?.is_some() && fs::exists(&folder.join(SKILL_MD))?;
            Ok(linked.then(|| (*name).to_owned()))
        }
        _ => Ok(None),
    }
}
