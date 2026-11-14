//! Pure counting: no I/O, every function is referentially transparent.

use core::fmt;
use core::iter::Sum;
use core::ops::Add;

/// File, line, and char totals for a set of files.
///
/// `Counts` is a monoid: [`Counts::default`] is the identity and `+` is associative, so
/// per-file counts can be folded in any grouping. Addition saturates instead of overflowing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[must_use]
pub struct Counts {
    text_files: usize,
    binary_files: usize,
    lines: usize,
    chars: usize,
}

impl Counts {
    /// Counts one text file.
    ///
    /// Lines follow [`str::lines`]: a final line without a trailing newline still counts.
    /// Chars are Unicode scalar values, not bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// let counts = dummy::Counts::of_text("héllo\nworld\n");
    /// assert_eq!(counts.lines(), 2);
    /// assert_eq!(counts.chars(), 12);
    /// ```
    pub fn of_text(text: &str) -> Self {
        Self {
            text_files: 1,
            binary_files: 0,
            lines: text.lines().count(),
            chars: text.chars().count(),
        }
    }

    /// Counts one binary file: it adds to the file total but has no lines or chars.
    pub const fn of_binary() -> Self {
        Self {
            text_files: 0,
            binary_files: 1,
            lines: 0,
            chars: 0,
        }
    }

    /// Counts one file from its raw content: valid UTF-8 is text, anything else is binary.
    ///
    /// # Examples
    ///
    /// ```
    /// use dummy::Counts;
    ///
    /// assert_eq!(Counts::of_bytes(b"a\nb"), Counts::of_text("a\nb"));
    /// assert_eq!(Counts::of_bytes(&[0xff, 0xfe]), Counts::of_binary());
    /// ```
    pub fn of_bytes(bytes: &[u8]) -> Self {
        core::str::from_utf8(bytes).map_or_else(|_| Self::of_binary(), Self::of_text)
    }

    /// Total number of files, text and binary.
    #[must_use]
    pub const fn files(&self) -> usize {
        self.text_files.saturating_add(self.binary_files)
    }

    /// Number of files whose content is valid UTF-8.
    #[must_use]
    pub const fn text_files(&self) -> usize {
        self.text_files
    }

    /// Number of files whose content is not valid UTF-8.
    #[must_use]
    pub const fn binary_files(&self) -> usize {
        self.binary_files
    }

    /// Number of lines across all text files.
    #[must_use]
    pub const fn lines(&self) -> usize {
        self.lines
    }

    /// Number of chars (Unicode scalar values) across all text files.
    #[must_use]
    pub const fn chars(&self) -> usize {
        self.chars
    }
}

impl Add for Counts {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            text_files: self.text_files.saturating_add(other.text_files),
            binary_files: self.binary_files.saturating_add(other.binary_files),
            lines: self.lines.saturating_add(other.lines),
            chars: self.chars.saturating_add(other.chars),
        }
    }
}

impl Sum for Counts {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Add::add)
    }
}

impl fmt::Display for Counts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "files: {} (text: {}, binary: {})",
            self.files(),
            self.text_files,
            self.binary_files
        )?;
        writeln!(f, "lines: {}", self.lines)?;
        write!(f, "chars: {}", self.chars)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "`Counts` addition saturates by construction"
)]
mod tests {
    use super::Counts;
    use proptest::prelude::{Just, Strategy, any, prop_assert_eq, prop_oneof, proptest};

    fn counts() -> impl Strategy<Value = Counts> {
        prop_oneof![
            any::<String>().prop_map(|text| Counts::of_text(&text)),
            Just(Counts::of_binary()),
        ]
    }

    #[test]
    fn empty_text_is_one_file_without_lines() {
        let counts = Counts::of_text("");
        assert_eq!((counts.files(), counts.lines(), counts.chars()), (1, 0, 0));
    }

    #[test]
    fn final_line_without_newline_counts() {
        assert_eq!(Counts::of_text("a\nb").lines(), 2);
        assert_eq!(Counts::of_text("a\nb\n").lines(), 2);
        assert_eq!(Counts::of_text("a\r\nb\r\n").lines(), 2);
    }

    #[test]
    fn chars_are_scalar_values_not_bytes() {
        assert_eq!(Counts::of_text("é🦀").chars(), 2);
    }

    #[test]
    fn invalid_utf8_is_binary() {
        let counts = Counts::of_bytes(&[b'a', 0xff]);
        assert_eq!(counts, Counts::of_binary());
        assert_eq!(
            (counts.files(), counts.binary_files(), counts.text_files()),
            (1, 1, 0)
        );
    }

    #[test]
    fn addition_saturates() {
        let full = Counts {
            text_files: usize::MAX,
            binary_files: 0,
            lines: usize::MAX,
            chars: usize::MAX,
        };
        let total = full + Counts::of_text("x") + Counts::of_binary();
        assert_eq!(
            (total.text_files(), total.lines(), total.chars()),
            (usize::MAX, usize::MAX, usize::MAX)
        );
        assert_eq!(total.files(), usize::MAX);
    }

    #[test]
    fn display_lists_every_total() {
        let total = Counts::of_text("one\ntwo\n") + Counts::of_binary();
        assert_eq!(
            total.to_string(),
            "files: 2 (text: 1, binary: 1)\nlines: 2\nchars: 8"
        );
    }

    proptest! {
        #[test]
        fn default_is_the_identity(counts in counts()) {
            prop_assert_eq!(counts + Counts::default(), counts);
            prop_assert_eq!(Counts::default() + counts, counts);
        }

        #[test]
        fn addition_is_associative(a in counts(), b in counts(), c in counts()) {
            prop_assert_eq!((a + b) + c, a + (b + c));
        }

        #[test]
        fn sum_matches_a_left_fold(all in proptest::collection::vec(counts(), 0..8)) {
            let folded = all.iter().fold(Counts::default(), |total, &next| total + next);
            prop_assert_eq!(all.into_iter().sum::<Counts>(), folded);
        }

        #[test]
        fn lines_and_chars_split_at_newlines(head in any::<String>(), tail in any::<String>()) {
            let head = format!("{head}\n");
            let whole = Counts::of_text(&format!("{head}{tail}"));
            let parts = Counts::of_text(&head) + Counts::of_text(&tail);
            prop_assert_eq!((whole.lines(), whole.chars()), (parts.lines(), parts.chars()));
        }
    }
}
