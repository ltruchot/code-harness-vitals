//! Relative Markdown links in a skill, and where they point: pure, no I/O.
//!
//! Both agents tell the model that paths in a skill are relative to the skill's directory
//! (Agent Skills spec, "File references"; `OpenCode` `skill.ts`, "Base directory for this
//! skill"). Links inside fenced code blocks and inline code are examples, not references.

use std::path::{Component, Path, PathBuf};

/// The local targets of every inline Markdown link (`[text](target)`) outside code, with
/// any `#fragment` or `?query` removed, in file order.
pub(crate) fn targets(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .scan(false, |fenced, line| {
            let fence =
                line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~");
            let prose = !*fenced && !fence;
            *fenced ^= fence;
            Some(prose.then_some(line))
        })
        .flatten()
        .flat_map(outside_code)
        .flat_map(link_targets)
        .filter_map(local)
        .collect()
}

/// The parts of `line` outside inline code spans.
fn outside_code(line: &str) -> impl Iterator<Item = &str> {
    line.split('`').step_by(2)
}

fn link_targets(text: &str) -> impl Iterator<Item = &str> {
    text.match_indices("](").filter_map(move |(at, _)| {
        let after = text.get(at.saturating_add(2)..)?;
        after.find(')').and_then(|end| after.get(..end))
    })
}

/// The path part of a link target when it is a local, relative path.
fn local(target: &str) -> Option<String> {
    let target = target.trim();
    let path = target.strip_prefix('<').map_or_else(
        || target.split_whitespace().next().unwrap_or_default(),
        |inner| inner.split('>').next().unwrap_or_default(),
    );
    let path = path.split(['#', '?']).next().unwrap_or_default();
    let remote = path.contains("://") || path.starts_with("mailto:") || path.starts_with('/');
    (!path.is_empty() && !remote).then(|| path.to_owned())
}

/// `target` resolved from `dir`, both relative to the repository root, without touching
/// the disk; `None` when it climbs out of the repository.
pub(crate) fn resolve(dir: &Path, target: &str) -> Option<PathBuf> {
    dir.join(target)
        .components()
        .try_fold(PathBuf::new(), |path, part| match part {
            Component::Normal(name) => Some(path.join(name)),
            Component::ParentDir => path.parent().map(Path::to_path_buf),
            Component::CurDir | Component::RootDir | Component::Prefix(_) => Some(path),
        })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{resolve, targets};
    use std::path::{Path, PathBuf};

    #[test]
    fn targets_keep_local_links_outside_code() {
        let markdown = "See [the guide](references/guide.md#setup) and [api](<api.md> \"API\").\n\
             Run `[not](a-link.md)` or [web](https://example.com), [mail](mailto:x), [top](#top).\n\
             ```md\n[example](missing.md)\n```\n\
             [abs](/etc/passwd) [query](scripts/run.sh?x=1) [empty]()\n";
        assert_eq!(
            targets(markdown),
            ["references/guide.md", "api.md", "scripts/run.sh"]
        );
    }

    #[test]
    fn tilde_fences_hide_links_too() {
        assert_eq!(targets("~~~\n[x](y.md)\n~~~\n"), Vec::<String>::new());
        assert_eq!(targets("~~~\n~~~\n[x](y.md)\n"), ["y.md"]);
    }

    #[test]
    fn resolve_normalizes_and_stays_in_the_repository() {
        let dir = Path::new(".claude/skills/deploy");
        assert_eq!(
            resolve(dir, "./references/../SKILL.md"),
            Some(PathBuf::from(".claude/skills/deploy/SKILL.md"))
        );
        assert_eq!(
            resolve(dir, "../../../docs/a.md"),
            Some(PathBuf::from("docs/a.md"))
        );
        assert_eq!(resolve(dir, "../../../../outside.md"), None);
    }
}
