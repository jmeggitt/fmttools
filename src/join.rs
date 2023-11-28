use std::cell::Cell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

pub struct Join<I, S> {
    iter: Cell<Option<I>>,
    separator: S,
}

#[inline]
pub fn join<I: IntoIterator, S: Display>(iter: I, separator: S) -> Join<I::IntoIter, S> {
    Join {
        iter: Cell::new(Some(iter.into_iter())),
        separator,
    }
}

impl<I, S> Debug for Join<I, S>
where
    I: Iterator,
    <I as Iterator>::Item: Debug,
    S: Display,
{
    #[inline]
    #[track_caller]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut item_iter = match self.iter.take() {
            Some(value) => value,
            None => panic!("Join can only be used once"),
        };

        let mut previous_item = match item_iter.next() {
            Some(value) => value,
            None => return Ok(()),
        };

        loop {
            match item_iter.next() {
                Some(next_item) => {
                    <I::Item as Debug>::fmt(&previous_item, f)?;
                    <S as Display>::fmt(&self.separator, f)?;
                    previous_item = next_item;
                }
                None => return <I::Item as Debug>::fmt(&previous_item, f),
            }
        }
    }
}

impl<I, S> Display for Join<I, S>
where
    I: Iterator,
    <I as Iterator>::Item: Display,
    S: Display,
{
    #[inline]
    #[track_caller]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut item_iter = match self.iter.take() {
            Some(value) => value,
            None => panic!("Join can only be used once"),
        };

        let mut previous_item = match item_iter.next() {
            Some(value) => value,
            None => return Ok(()),
        };

        loop {
            match item_iter.next() {
                Some(next_item) => {
                    <I::Item as Display>::fmt(&previous_item, f)?;
                    <S as Display>::fmt(&self.separator, f)?;
                    previous_item = next_item;
                }
                None => return <I::Item as Display>::fmt(&previous_item, f),
            }
        }
    }
}

#[inline]
pub fn join_fmt<I, F, S>(iter: I, fmt_item: F, separator: S) -> JoinFmt<I::IntoIter, F, S>
where
    I: IntoIterator,
    F: FnMut(I::Item, &mut Formatter<'_>) -> fmt::Result,
    S: Display,
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

#[inline]
pub fn join_fmt_all<I, F, S>(
    iter: I,
    fmt_item: F,
    fmt_separator: S,
) -> JoinFmtAll<I::IntoIter, F, S>
where
    I: IntoIterator,
    F: FnMut(I::Item, &mut Formatter<'_>) -> fmt::Result,
    S: FnMut(&mut Formatter<'_>) -> fmt::Result,
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
