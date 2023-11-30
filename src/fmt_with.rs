//! This module provides the boilerplate to implement [std::fmt::Debug] and [std::fmt::Display] for
//! cases where additional information is required to properly format a type.
//!
//! ```rust
//! # use std::collections::HashMap;
//! # use std::fmt::{self, Formatter};
//! use fmttools::{DebugWith, ToFormatWith};
//!
//! type RegistryKey = u32;
//!
//! struct Registry {
//!     key_names: HashMap<RegistryKey, String>,
//! }
//!
//! struct FooEntry {
//!     key: RegistryKey,
//! }
//!
//! impl DebugWith<Registry> for FooEntry {
//!     fn fmt(&self, f: &mut Formatter<'_>, registry: &Registry) -> fmt::Result {
//!         let key_name = registry.key_names.get(&self.key)
//!             .map(|x| x.as_str())
//!             .unwrap_or("unknown");
//!
//!         write!(f, "FooEntry {{ key: {:?} }}", key_name)
//!     }
//! }
//!
//! let registry = Registry {
//!     key_names: HashMap::from([
//!         (2, "FooA".to_string()),
//!         (5, "FooB".to_string()),
//!         (9, "Bar".to_string()),
//!     ]),
//! };
//!
//! let entry = FooEntry { key: 5 };
//!
//! assert_eq!("FooEntry { key: \"FooB\" }", format!("{:?}", entry.fmt_with(&registry)));
//! ```
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

/// See [crate::fmt_with] for more information.
pub trait ToFormatWith<T> {
    fn fmt_with<'a>(&'a self, extra: &'a T) -> FormatWith<'a, Self, T>;
}

impl<T, E> ToFormatWith<E> for T {
    #[inline]
    fn fmt_with<'a>(&'a self, extra: &'a E) -> FormatWith<'a, Self, E> {
        FormatWith { this: self, extra }
    }
}

/// See [crate::fmt_with] for more information.
pub struct FormatWith<'a, T: ?Sized, E: ?Sized> {
    this: &'a T,
    extra: &'a E,
}

/// See [crate::fmt_with] for more information.
pub trait DisplayWith<T> {
    fn fmt(&self, f: &mut Formatter<'_>, extra: &T) -> fmt::Result;
}

impl<T: DisplayWith<E>, E> Display for FormatWith<'_, T, E> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.this.fmt(f, self.extra)
    }
}

/// See [crate::fmt_with] for more information.
pub trait DebugWith<T: ?Sized> {
    fn fmt(&self, f: &mut Formatter<'_>, extra: &T) -> fmt::Result;
}

impl<T: DebugWith<E>, E> Debug for FormatWith<'_, T, E> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.this.fmt(f, self.extra)
    }
}
