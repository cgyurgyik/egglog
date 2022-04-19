use crate::util::*;

use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;
use std::sync::RwLock;

use once_cell::sync::Lazy;

static STRINGS: Lazy<RwLock<IndexSet<&'static str>>> = Lazy::new(Default::default);

/// An interned string.
///
/// Internally, `egg` frequently compares [`Var`]s and elements of
/// [`Language`]s. To keep comparisons fast, `egg` provides [`Symbol`] a simple
/// wrapper providing interned strings.
///
/// You may wish to use [`Symbol`] in your own [`Language`]s to increase
/// performance and keep enode sizes down (a [`Symbol`] is only 4 bytes,
/// compared to 24 for a `String`.)
///
/// A [`Symbol`] is simply a wrapper around an integer.
/// When creating a [`Symbol`] from a string, `egg` looks up it up in a global
/// table, returning the index (inserting it if not found).
/// That integer is used to cheaply implement
/// `Copy`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash`.
///
/// The internal symbol cache leaks the strings, which should be
/// fine if you only put in things like variable names and identifiers.
///
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde-1", serde(from = "&str", into = "&'static str"))]
pub struct Symbol(u32);

impl Symbol {
    /// Get the string that this symbol represents
    pub fn as_str(self) -> &'static str {
        let i = self.0 as usize;
        let strings = STRINGS
            .read()
            .unwrap_or_else(|err| panic!("Failed to acquire egg's global string cache: {}", err));
        strings.get_index(i).unwrap()
    }
}

fn leak(s: &str) -> &'static str {
    Box::leak(s.to_owned().into_boxed_str())
}

fn intern(s: &str) -> Symbol {
    let strings = STRINGS
        .read()
        .unwrap_or_else(|err| panic!("Failed to acquire egg's global string cache: {}", err));
    if let Some((i, _)) = strings.get_full(s) {
        return Symbol(i as u32);
    }
    // Release the read lock.
    drop(strings);

    let mut strings = STRINGS
        .write()
        .unwrap_or_else(|err| panic!("Failed to acquire egg's global string cache: {}", err));
    let i = match strings.get_full(s) {
        Some((i, _)) => i, // The string was inserted in the meantime.
        None => strings.insert_full(leak(s)).0,
    };
    Symbol(i as u32)
}

impl<S: AsRef<str>> From<S> for Symbol {
    fn from(s: S) -> Self {
        intern(s.as_ref())
    }
}

impl From<Symbol> for &'static str {
    fn from(s: Symbol) -> Self {
        s.as_str()
    }
}

impl FromStr for Symbol {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

/// A wrapper that uses display implementation as debug
pub(crate) struct DisplayAsDebug<T>(pub T);

impl<T: Display> Debug for DisplayAsDebug<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}