//! `SKILL.md` frontmatter, read by a line scanner: pure, no I/O.
//!
//! This is not a YAML parser (ADR 4, "The skills gate"). It finds the top-level keys of the
//! block between the opening `---` (first line) and the closing `---`, reads each key's
//! scalar value (plain, quoted, or block), and flags the two plain-value mistakes that YAML
//! rejects: a `: ` inside the value, and a value that opens with a reserved indicator or a
//! flow collection followed by text. Claude Code then loads the skill with no fields;
//! `OpenCode` (`config/markdown.ts`) retries values that contain `:` or start with
//! `[x] text` as block scalars, and skips the skill otherwise.

/// What a top-level key holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
    /// A string scalar.
    Text(String),
    /// No value: YAML null.
    Empty,
    /// A list or a map: not a string to either agent.
    Collection,
}

/// Why YAML rejects a plain value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mistake {
    /// `: ` inside the value, or a trailing `:`: `OpenCode` repairs it.
    Colon,
    /// `[x] text`: `OpenCode` repairs it.
    TaggedText,
    /// Opens with `` ` `` or `@`, which YAML reserves: `OpenCode` repairs it only when the
    /// value also holds a `:`.
    Reserved { repaired: bool },
}

impl Mistake {
    /// Whether `OpenCode`'s retry turns the value back into a string.
    pub(crate) const fn repaired(self) -> bool {
        match self {
            Self::Colon | Self::TaggedText => true,
            Self::Reserved { repaired } => repaired,
        }
    }

    /// What is wrong, for a finding message.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::Colon => "holds an unquoted `: `",
            Self::TaggedText => "opens with `[...]` followed by text",
            Self::Reserved { .. } => "opens with `` ` `` or `@`, which YAML reserves",
        }
    }
}

/// One top-level key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Field {
    pub(crate) key: String,
    pub(crate) value: Value,
    /// The key's line in the file, from 1.
    pub(crate) line: usize,
    pub(crate) mistake: Option<Mistake>,
}

/// The frontmatter of a `SKILL.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Frontmatter {
    /// The first line is not `---`.
    Missing,
    /// The first line is `---` and no later line closes the block.
    Unclosed,
    /// The fields, in file order.
    Fields(Vec<Field>),
}

impl Frontmatter {
    /// The field `key`, when the block has it.
    pub(crate) fn field(&self, key: &str) -> Option<&Field> {
        match self {
            Self::Fields(fields) => fields.iter().find(|field| field.key == key),
            Self::Missing | Self::Unclosed => None,
        }
    }

    /// The string value of `key`, trimmed; `None` when absent, empty, or not a string.
    pub(crate) fn text(&self, key: &str) -> Option<&str> {
        self.field(key).and_then(|field| match &field.value {
            Value::Text(text) => Some(text.trim()).filter(|text| !text.is_empty()),
            Value::Empty | Value::Collection => None,
        })
    }

    /// Whether `key` is a true boolean, as both agents read one (`true`, `yes`, `on`, `1`,
    /// in any letter case).
    pub(crate) fn is_true(&self, key: &str) -> bool {
        self.text(key).is_some_and(|text| {
            ["true", "yes", "on", "1"]
                .iter()
                .any(|truth| text.eq_ignore_ascii_case(truth))
        })
    }
}

const FENCE: &str = "---";

/// Reads the frontmatter of `text`.
pub(crate) fn read(text: &str) -> Frontmatter {
    let mut lines = text.lines();
    if lines.next().map(str::trim_end) != Some(FENCE) {
        return Frontmatter::Missing;
    }
    let block: Vec<&str> = lines
        .by_ref()
        .take_while(|line| line.trim_end() != FENCE)
        .collect();
    // `take_while` consumed the closing fence when there was one; the block then ends
    // before the end of the text.
    let closed = text.lines().skip(1).any(|line| line.trim_end() == FENCE);
    if closed {
        Frontmatter::Fields(entries(&block).iter().map(Entry::field).collect())
    } else {
        Frontmatter::Unclosed
    }
}

/// A top-level key line and the lines that continue its value.
struct Entry<'a> {
    line: usize,
    key: &'a str,
    inline: &'a str,
    rest: Vec<&'a str>,
}

fn entries<'a>(block: &[&'a str]) -> Vec<Entry<'a>> {
    block
        .iter()
        .enumerate()
        .fold(Vec::new(), |mut found, (index, line)| {
            if let Some((key, inline)) = key_line(line) {
                found.push(Entry {
                    // The block starts on the file's second line.
                    line: index.saturating_add(2),
                    key,
                    inline,
                    rest: Vec::new(),
                });
            } else if let Some(last) = found.last_mut() {
                last.rest.push(line);
            }
            found
        })
}

/// `key: value` at column 0: a key of letters, digits, `_`, or `-`, then `:` and a space or
/// the end of the line.
fn key_line(line: &str) -> Option<(&str, &str)> {
    let (key, after) = line.split_once(':')?;
    let is_key = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    let separated = after.is_empty() || after.starts_with([' ', '\t']);
    (is_key && separated).then(|| (key, after.trim()))
}

impl Entry<'_> {
    fn field(&self) -> Field {
        let (value, mistake) = self.value();
        Field {
            key: self.key.to_owned(),
            value,
            line: self.line,
            mistake,
        }
    }

    fn continuation(&self) -> impl Iterator<Item = &str> {
        self.rest
            .iter()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
    }

    fn value(&self) -> (Value, Option<Mistake>) {
        let inline = self.inline;
        if inline.is_empty() {
            return (self.nested(), None);
        }
        if inline.starts_with(['|', '>']) {
            return (Value::Text(self.block(inline)), None);
        }
        if let Some(quoted) = quoted(inline) {
            return (Value::Text(quoted), None);
        }
        if inline.starts_with(['[', '{']) {
            return flow(inline);
        }
        let text = core::iter::once(plain(inline))
            .chain(self.continuation())
            .collect::<Vec<_>>()
            .join(" ");
        (Value::Text(text), plain_mistake(plain(inline)))
    }

    /// A block scalar: `|` keeps line breaks, `>` folds them into spaces.
    fn block(&self, indicator: &str) -> String {
        let separator = if indicator.starts_with('|') {
            "\n"
        } else {
            " "
        };
        self.continuation().collect::<Vec<_>>().join(separator)
    }

    /// A value on the lines below the key: a list, a map, or a multi-line plain scalar.
    fn nested(&self) -> Value {
        let lines: Vec<&str> = self.continuation().collect();
        match lines.first() {
            None => Value::Empty,
            Some(first) if first.starts_with('-') || key_line(first).is_some() => Value::Collection,
            Some(_) => Value::Text(lines.join(" ")),
        }
    }
}

/// A single- or double-quoted scalar on one line, unescaped.
fn quoted(inline: &str) -> Option<String> {
    let quote = inline.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let inner = inline.strip_prefix(quote)?;
    let end = inner.rfind(quote).unwrap_or(inner.len());
    let body = inner.get(..end).unwrap_or(inner);
    Some(if quote == '"' {
        body.replace("\\\"", "\"").replace("\\\\", "\\")
    } else {
        body.replace("''", "'")
    })
}

/// `[a, b]` or `{a: b}` is a collection; `[Tag] text` is a YAML error that `OpenCode`
/// repairs into a string.
fn flow(inline: &str) -> (Value, Option<Mistake>) {
    let close = inline.rfind([']', '}']);
    let trailing = close
        .and_then(|at| inline.get(at.saturating_add(1)..))
        .is_some_and(|after| !after.trim().is_empty() && !after.trim().starts_with('#'));
    if trailing {
        (Value::Text(inline.to_owned()), Some(Mistake::TaggedText))
    } else {
        (Value::Collection, None)
    }
}

/// The plain value without a trailing ` # comment`.
fn plain(inline: &str) -> &str {
    inline
        .find(" #")
        .and_then(|at| inline.get(..at))
        .unwrap_or(inline)
        .trim_end()
}

fn plain_mistake(value: &str) -> Option<Mistake> {
    let colon = value.contains(": ") || value.ends_with(':');
    if value.starts_with(['`', '@']) {
        Some(Mistake::Reserved {
            repaired: value.contains(':'),
        })
    } else {
        colon.then_some(Mistake::Colon)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Frontmatter, Mistake, Value, read};

    fn fields(text: &str) -> Frontmatter {
        let found = read(text);
        assert!(matches!(found, Frontmatter::Fields(_)), "{found:?}");
        found
    }

    #[test]
    fn missing_and_unclosed_blocks() {
        assert_eq!(read("# Title\n---\nname: x\n---\n"), Frontmatter::Missing);
        assert_eq!(read(""), Frontmatter::Missing);
        assert_eq!(read("---\nname: x\n"), Frontmatter::Unclosed);
        assert_eq!(read("---\n---\n"), Frontmatter::Fields(Vec::new()));
        assert_eq!(read("---\r\nname: x\r\n---\r\n").text("name"), Some("x"));
    }

    #[test]
    fn plain_quoted_and_block_values() {
        let found = fields(
            "---\nname: deploy # the id\ndescription: \"Ship it: \\\"now\\\"\"\n\
             single: 'it''s'\nfolded: >-\n  one\n  two\nliteral: |\n  a\n  b\n\
             wrapped: first\n  second\nempty:\n---\nbody\n",
        );
        assert_eq!(found.text("name"), Some("deploy"));
        assert_eq!(found.text("description"), Some("Ship it: \"now\""));
        assert_eq!(found.text("single"), Some("it's"));
        assert_eq!(found.text("folded"), Some("one two"));
        assert_eq!(found.text("literal"), Some("a\nb"));
        assert_eq!(found.text("wrapped"), Some("first second"));
        assert_eq!(found.field("empty").unwrap().value, Value::Empty);
        assert_eq!(found.text("empty"), None);
        assert_eq!(found.field("description").unwrap().line, 3);
        assert_eq!(found.text("absent"), None);
    }

    #[test]
    fn lists_and_maps_are_collections() {
        let found =
            fields("---\ntags: [a, b]\nmeta: {k: v}\nlist:\n- a\n- b\nmap:\n  key: value\n---\n");
        for key in ["tags", "meta", "list", "map"] {
            assert_eq!(found.field(key).unwrap().value, Value::Collection, "{key}");
            assert_eq!(found.text(key), None);
        }
    }

    #[test]
    fn yaml_mistakes_in_plain_values() {
        let found = fields(
            "---\ndescription: Use when: deploying\nname: x:\nwhen: [Beta] fast\n\
             a: `code` here\nb: @handle: x\nurl: https://example.com\nnote: x # a: b\n---\n",
        );
        let mistake = |key| found.field(key).unwrap().mistake;
        assert_eq!(mistake("description"), Some(Mistake::Colon));
        assert_eq!(mistake("name"), Some(Mistake::Colon));
        assert_eq!(mistake("when"), Some(Mistake::TaggedText));
        assert_eq!(found.text("when"), Some("[Beta] fast"));
        assert_eq!(mistake("a"), Some(Mistake::Reserved { repaired: false }));
        assert_eq!(mistake("b"), Some(Mistake::Reserved { repaired: true }));
        assert_eq!(mistake("url"), None);
        assert_eq!(mistake("note"), None);
        assert!(Mistake::Colon.repaired());
        assert!(!Mistake::Reserved { repaired: false }.repaired());
        assert!(Mistake::TaggedText.reason().contains("[...]"));
    }

    #[test]
    fn booleans_read_like_both_agents() {
        let found = fields("---\na: true\nb: Yes\nc: on\nd: 1\ne: false\nf: [true]\n---\n");
        assert!(["a", "b", "c", "d"].iter().all(|key| found.is_true(key)));
        assert!(!found.is_true("e"));
        assert!(!found.is_true("f"));
        assert!(!found.is_true("absent"));
        assert!(!Frontmatter::Missing.is_true("a"));
    }

    #[test]
    fn lines_that_are_not_keys_continue_the_previous_value() {
        let found = fields("---\n# comment\nstray line\nhttp://x: y\nname: x\n  # note\n---\n");
        assert_eq!(found.text("name"), Some("x"));
        assert_eq!(found.field("http"), None);
    }
}
