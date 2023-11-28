use std::fmt;
use std::fmt::{Debug, Display, Formatter, Write};

#[inline]
pub fn replace<T, P>(value: T, pattern: P, replacement: &str) -> Replace<T, P> {
    Replace {
        value,
        pattern,
        replacement,
    }
}

pub struct Replace<'a, T, P> {
    value: T,
    pattern: P,
    replacement: &'a str,
}

impl<'a, T, P> Debug for Replace<'a, T, P>
where
    T: Debug,
    P: ReplacePattern,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.pattern
            .fmt_impl(self.replacement, f, |out| write!(out, "{:?}", self.value))
    }
}

impl<'a, T, P> Display for Replace<'a, T, P>
where
    T: Display,
    P: ReplacePattern,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.pattern
            .fmt_impl(self.replacement, f, |out| write!(out, "{}", self.value))
    }
}

pub trait ReplacePattern {
    fn fmt_impl<W, F>(&self, replacement: &str, out: W, func: F) -> fmt::Result
    where
        W: Write,
        F: FnOnce(&mut dyn Write) -> fmt::Result;
}

pub struct CharReplacer<'a, W> {
    dst: W,
    pattern: char,
    replacement: &'a str,
}

impl<'a, W: Write> Write for CharReplacer<'a, W> {
    #[inline]
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while let Some(index) = s.find(self.pattern) {
            self.dst.write_str(&s[..index])?;
            self.dst.write_str(self.replacement)?;
            s = &s[index + self.pattern.len_utf8()..];
        }

        self.dst.write_str(s)
    }
}

impl ReplacePattern for char {
    #[inline]
    fn fmt_impl<W, F>(&self, replacement: &str, out: W, func: F) -> fmt::Result
    where
        W: Write,
        F: FnOnce(&mut dyn Write) -> fmt::Result,
    {
        let mut writer = CharReplacer {
            dst: out,
            pattern: *self,
            replacement,
        };
        func(&mut writer)
    }
}

// On drop use self.dst.write_str(&self.pattern[..self.withheld])
pub struct StrReplacer<'a, W> {
    dst: W,
    /// Pattern to be matched
    pattern: &'a str,
    /// Replacement to be used when pattern is discovered
    replacement: &'a str,
    /// How many bytes of the pattern were withheld due to the possibility of a match
    withheld: usize,
}

impl<'a, W> StrReplacer<'a, W> {
    /// We failed the match using the current number of withheld bytes. Find the next smallest
    /// number of withheld bytes that maintains our requirements.
    #[inline]
    fn backoff(&self) -> usize {
        if self.withheld < 2 {
            return 0;
        }

        for offset in 0..self.withheld {
            if !self.pattern.is_char_boundary(offset) {
                continue;
            }

            if self.pattern[..self.withheld - offset] == self.pattern[offset..self.withheld] {
                return self.withheld - offset;
            }
        }

        0
    }
}

impl<'a, W: Write> Write for StrReplacer<'a, W> {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        let first_char = match self.pattern.chars().next() {
            None => unreachable!("is_empty was checked when constructing replacer"),
            Some(c) => c,
        };

        while !s.is_empty() {
            if self.withheld == 0 {
                // Find the pattern and move until we withold at least 1 character
                match s.find(first_char) {
                    None => return self.dst.write_str(s),
                    Some(index) => {
                        self.dst.write_str(&s[..index])?;
                        self.withheld = first_char.len_utf8();
                        s = &s[index + first_char.len_utf8()..];
                        continue;
                    }
                }
            }

            // Greedily attempt to match as much as possible
            let overlap_len = s.len().min(self.pattern.len() - self.withheld);
            if s.as_bytes()[..overlap_len] == self.pattern.as_bytes()[self.withheld..] {
                self.withheld += overlap_len;
                s = &s[overlap_len..];
                if self.withheld == self.pattern.len() {
                    self.withheld = 0;
                    self.dst.write_str(self.replacement)?;
                }
                continue;
            }

            // We failed the greedy match
            let new_withheld = self.backoff();
            self.dst
                .write_str(&self.pattern[..self.withheld - new_withheld])?;
            self.withheld = new_withheld;
        }

        Ok(())
    }
}

struct EmptyReplacer<'a, W> {
    out: W,
    replacement: &'a str,
}

impl<'a, W: Write> Write for EmptyReplacer<'a, W> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.out.write_str(self.replacement)?;
            self.out.write_char(c)?;
        }

        Ok(())
    }
}

impl<'a> ReplacePattern for &'a str {
    #[inline]
    fn fmt_impl<W, F>(&self, replacement: &str, out: W, func: F) -> fmt::Result
    where
        W: Write,
        F: FnOnce(&mut dyn Write) -> fmt::Result,
    {
        let mut length_check = self.chars();
        let first = length_check.next();
        let has_additional = first.is_some() && length_check.next().is_some();

        match (first, has_additional) {
            // The pattern is an empty string
            (None, _) => {
                let mut replacer = EmptyReplacer { out, replacement };
                func(&mut replacer)?;
                replacer.out.write_str(replacement)
            }
            // We are replacing a 1 character string so defer to char pattern
            (Some(pattern), false) => pattern.fmt_impl(replacement, out, func),
            // We have more than 2 characters so use the regular approach
            (Some(_), true) => {
                let mut writer = StrReplacer {
                    dst: out,
                    pattern: self,
                    replacement,
                    withheld: 0,
                };
                func(&mut writer)?;

                if writer.withheld == writer.pattern.len() {
                    writer.dst.write_str(writer.replacement)
                } else {
                    writer.dst.write_str(&writer.pattern[..writer.withheld])
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::replace;

    #[test]
    fn replace_char_simple() {
        let out = format!("{}", replace(".abc. defs ... fd.", '.', "foo"));
        assert_eq!(out, ".abc. defs ... fd.".replace('.', "foo"));
    }

    #[test]
    fn replace_char_same() {
        let out = format!("{}", replace(".abc. defs ... fd.", '.', "."));
        assert_eq!(out, ".abc. defs ... fd.".replace('.', "."));
    }

    #[test]
    fn replace_char_complex() {
        let out = format!(
            "{}",
            replace(
                format_args!("{} . abc{:?} {}", 13.25, "a.b", ".."),
                '.',
                "foo"
            )
        );
        let expected = "13foo25 foo abc\"afoob\" foofoo";

        assert_eq!(out, expected);
    }

    #[test]
    fn replace_str_same() {
        let out = format!("{}", replace(".abc. defs aaab...aba fda", "ab", "ab"));
        assert_eq!(out, ".abc. defs aaab...aba fda".replace("ab", "ab"));
    }

    #[test]
    fn replace_1_letter_str() {
        let out = format!("{}", replace(".abc. defs aaab...aba fda", "a", "bc"));
        assert_eq!(out, ".abc. defs aaab...aba fda".replace("a", "bc"));
    }

    #[test]
    fn replace_2_letter_str() {
        let out = format!("{}", replace(".abc. defs aaab...aba fda", "ab", "bc"));
        assert_eq!(out, ".abc. defs aaab...aba fda".replace("ab", "bc"));
    }

    #[test]
    fn replace_5_letter_str() {
        let out = format!("{}", replace("abcdfewdabdwfeabcd", "abcdf", "fdcba"));
        assert_eq!(out, "abcdfewdabdwfeabcd".replace("abcdf", "fdcba"));
    }

    #[test]
    fn replace_str_with_backtrack() {
        let out = format!(
            "{}",
            replace("aaafafaffafafaaaaaaafaaaafaaaffaaaaf", "aaaaf", "123")
        );
        assert_eq!(
            out,
            "aaafafaffafafaaaaaaafaaaafaaaffaaaaf".replace("aaaaf", "123")
        );
    }

    #[test]
    fn replace_str_with_backtrack_2() {
        let out = format!("{}", replace("abacaababababccabcaabbaccab", "ababc", "123"));
        assert_eq!(out, "abacaababababccabcaabbaccab".replace("ababc", "123"));
    }

    #[test]
    fn replace_empty_str() {
        let out = format!("{}", replace("abcdefg", "", "."));
        assert_eq!(out, "abcdefg".replace("", "."));
    }

    #[test]
    fn replace_str_complex() {
        let out = format!(
            "{}",
            replace(
                format_args!("{} . abc2{:?} {}2", 13.25, "a.b", ".."),
                ".2",
                "foo"
            )
        );
        let expected = format!("{} . abc2{:?} {}2", 13.25, "a.b", "..").replace(".2", "foo");

        assert_eq!(out, expected);
    }
}
