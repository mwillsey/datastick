use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Formatter},
    hash::Hash,
    num::NonZeroU32,
    sync::Mutex,
};

use once_cell::sync::Lazy;

pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V>;
pub(crate) type IndexSet<K> = indexmap::IndexSet<K>;

// pub(crate) type HashMap<K, V> = std::collections::HashMap<K, V>;
pub(crate) type HashSet<K> = std::collections::HashSet<K>;

static SYMBOLS: Lazy<Mutex<IndexSet<&'static str>>> = Lazy::new(Default::default);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(NonZeroU32);

impl Symbol {
    pub fn new<T>(s: T) -> Self
    where
        T: Borrow<str> + Into<Box<str>>,
    {
        let mut syms = SYMBOLS.lock().unwrap();
        let idx = syms
            .get_index_of(s.borrow())
            .unwrap_or_else(|| syms.insert_full(Box::leak(s.into())).0);
        let idx = NonZeroU32::new(idx as u32 + 1).unwrap();
        Symbol(idx)
    }
}

/// Shorthand for [Symbol::new]
pub fn sym<T>(s: T) -> Symbol
where
    T: Borrow<str> + Into<Box<str>>,
{
    Symbol::new(s)
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        let syms = SYMBOLS.lock().unwrap();
        let i = self.0.get() as usize - 1;
        syms.get_index(i).unwrap()
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_ref(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbols() {
        let foo = Symbol::new("foo");
        let foo2 = Symbol::new("fo".to_string() + "o");
        assert_eq!(foo.0, foo2.0);
        assert_eq!(format!("{}", foo), "foo");
        assert_eq!(format!("{:?}", foo), "foo");
    }
}
