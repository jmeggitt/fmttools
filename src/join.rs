use std::cell::Cell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

pub struct Join<'a, I> {
    iter: Cell<Option<I>>,
    separator: &'a str,
}

/// Joins iterator elements together with a given separator. Formatting is only performed during
/// [Debug::fmt] or [Display::fmt].
/// ```rust
/// use fmttools::join;
///
/// let elements = vec![1, 2, 3, 4, 5];
/// assert_eq!("1:2:3:4:5", format!("{}", join(&elements, ":")));
/// ```
///
/// ## Note
/// Elements are formatted according to either their debug or display implementations. Format string
/// arguments are not passed to elements.
/// ```rust
/// use fmttools::join;
///
/// let elements = vec!["abc", "\n", "123"];
/// assert_eq!("abc, \n, 123", format!("{}", join(&elements, ", ")));
/// assert_eq!("\"abc\", \"\\n\", \"123\"", format!("{:?}", join(&elements, ", ")));
/// ```
///
/// See [join_fmt] and [join_fmt_all] for additional control over element and separator formatting.
#[inline]
pub fn join<I: IntoIterator>(iter: I, separator: &str) -> Join<I::IntoIter> {
    Join {
        iter: Cell::new(Some(iter.into_iter())),
        separator,
    }
}

impl<I> Debug for Join<'_, I>
where
    I: Iterator,
    <I as Iterator>::Item: Debug,
{
    #[inline]
    #[track_caller]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut item_iter = match self.iter.take() {
            Some(value) => value,
            None => panic!("Join can only be used once"),
        };

        match item_iter.next() {
            Some(value) => <I::Item as Debug>::fmt(&value, f)?,
            None => return Ok(()),
        }

        for item in item_iter {
            f.write_str(self.separator)?;
            <I::Item as Debug>::fmt(&item, f)?;
        }

        Ok(())
    }
}

impl<I> Display for Join<'_, I>
where
    I: Iterator,
    <I as Iterator>::Item: Display,
{
    #[inline]
    #[track_caller]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut item_iter = match self.iter.take() {
            Some(value) => value,
            None => panic!("Join can only be used once"),
        };

        match item_iter.next() {
            Some(value) => <I::Item as Display>::fmt(&value, f)?,
            None => return Ok(()),
        }

        for item in item_iter {
            f.write_str(self.separator)?;
            <I::Item as Display>::fmt(&item, f)?;
        }

        Ok(())
    }
}

/// Joins iterator elements together with a given separator. Formatting is only performed during
/// [Debug::fmt] or [Display::fmt].
/// ```rust
/// # use std::fmt;
/// # use std::fmt::Formatter;
/// use fmttools::join_fmt;
///
/// // Alternatively, a closure can be used
/// fn format_element(x: &i32, f: &mut Formatter<'_>) -> fmt::Result {
///     if *x > 3 {
///         return write!(f, "3+");
///     }
///
///     write!(f, "{}", x)
/// }
///
/// let elements = vec![1, 2, 3, 4, 5];
/// assert_eq!("1, 2, 3, 3+, 3+", format!("{}", join_fmt(&elements, ", ", format_element)));
/// ```
/// See [join] to format elements according to their [Debug] or [Display] implementations. See
/// [join_fmt_all] for additional control over formatting element separators.
#[inline]
pub fn join_fmt<I, S, F>(iter: I, separator: S, fmt_item: F) -> JoinFmt<I::IntoIter, F, S>
where
    I: IntoIterator,
    S: Display,
    F: FnMut(I::Item, &mut Formatter<'_>) -> fmt::Result,
{
    let inner = JoinFmtInner {
        iter: iter.into_iter(),
        element_writer: fmt_item,
    };

    JoinFmt {
        inner: Cell::new(Some(inner)),
        separator,
    }
}

pub struct JoinFmt<I, F, S> {
    inner: Cell<Option<JoinFmtInner<I, F>>>,
    separator: S,
}

struct JoinFmtInner<I, F> {
    iter: I,
    element_writer: F,
}

impl<I, F, S> Display for JoinFmt<I, F, S>
where
    I: Iterator,
    F: FnMut(I::Item, &mut Formatter<'_>) -> fmt::Result,
    S: Display,
{
    #[inline]
    #[track_caller]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let JoinFmtInner {
            mut iter,
            mut element_writer,
        } = match self.inner.take() {
            Some(value) => value,
            None => panic!("Join can only be used once"),
        };

        let mut previous = match iter.next() {
            Some(value) => value,
            None => return Ok(()),
        };

        for next in iter {
            element_writer(previous, f)?;
            <S as Display>::fmt(&self.separator, f)?;
            previous = next;
        }

        element_writer(previous, f)
    }
}

/// Joins iterator elements together while formatting using the specified formatting functions for
/// elements and separators. Formatting is only performed during [Display::fmt].
/// ```rust
/// # use std::fmt;
/// # use std::fmt::Formatter;
/// use fmttools::join_fmt_all;
///
/// fn format_element(x: &i32, f: &mut Formatter<'_>) -> fmt::Result {
///     write!(f, "({})", x)
/// }
///
/// let mut positive = true;
/// let format_separator = |f: &mut Formatter<'_>| {
///     positive = !positive;
///     if positive {
///         write!(f, " + ")
///     } else {
///         write!(f, " - ")
///     }
/// };
///
/// let elements = vec![1, 2, 3, 4, 5];
/// assert_eq!("(1) - (2) + (3) - (4) + (5)", format!("{}", join_fmt_all(&elements, format_separator, format_element)));
/// ```
/// See [join] to format elements according to their [Debug] or [Display] implementations. See
/// [join_fmt] is separator format control is not required.
#[inline]
pub fn join_fmt_all<I, S, F>(
    iter: I,
    fmt_separator: S,
    fmt_item: F,
) -> JoinFmtAll<I::IntoIter, F, S>
where
    I: IntoIterator,
    S: FnMut(&mut Formatter<'_>) -> fmt::Result,
    F: FnMut(I::Item, &mut Formatter<'_>) -> fmt::Result,
{
    let inner = JoinFmtAllInner {
        iter: iter.into_iter(),
        element_writer: fmt_item,
        separator_writer: fmt_separator,
    };

    JoinFmtAll {
        inner: Cell::new(Some(inner)),
    }
}

pub struct JoinFmtAll<I, F, S> {
    inner: Cell<Option<JoinFmtAllInner<I, F, S>>>,
}

struct JoinFmtAllInner<I, F, S> {
    iter: I,
    element_writer: F,
    separator_writer: S,
}

impl<I, F, S> Display for JoinFmtAll<I, F, S>
where
    I: Iterator,
    F: FnMut(I::Item, &mut Formatter<'_>) -> fmt::Result,
    S: FnMut(&mut Formatter<'_>) -> fmt::Result,
{
    #[inline]
    #[track_caller]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Some(JoinFmtAllInner {
            mut iter,
            mut element_writer,
            mut separator_writer,
        }) = self.inner.take()
        else {
            panic!("Join can only be used once");
        };

        let Some(mut previous) = iter.next() else {
            return Ok(());
        };

        for next in iter {
            element_writer(previous, f)?;
            separator_writer(f)?;
            previous = next;
        }

        element_writer(previous, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::join::join;

    #[test]
    pub fn join_debug() {
        let values = ["abc", "def", "\0123"];

        let output = format!("{:?}", join(values, ", "));
        assert_eq!(output, "\"abc\", \"def\", \"\\0123\"");
    }

    #[test]
    pub fn join_display() {
        let values = ["abc", "def", "\0123"];

        let output = format!("{}", join(values, ", "));
        assert_eq!(output, "abc, def, \0123");
    }
}
