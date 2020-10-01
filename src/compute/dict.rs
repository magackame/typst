//! A key-value map that can also model array-like structures.

use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Index;

use crate::syntax::{Span, Spanned};

/// A dictionary data structure, which maps from integers (`u64`) or strings to
/// a generic value type.
///
/// The dictionary can be used to model arrays by assigning values to successive
/// indices from `0..n`. The `push` method offers special support for this
/// pattern.
#[derive(Clone)]
pub struct Dict<V> {
    nums: BTreeMap<u64, V>,
    strs: BTreeMap<String, V>,
    lowest_free: u64,
}

impl<V> Dict<V> {
    /// Create a new empty dictionary.
    pub fn new() -> Self {
        Self {
            nums: BTreeMap::new(),
            strs: BTreeMap::new(),
            lowest_free: 0,
        }
    }

    /// The total number of entries in the dictionary.
    pub fn len(&self) -> usize {
        self.nums.len() + self.strs.len()
    }

    /// Whether the dictionary contains no entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The first number key-value pair (with lowest number).
    pub fn first(&self) -> Option<(u64, &V)> {
        self.nums.iter().next().map(|(&k, v)| (k, v))
    }

    /// The last number key-value pair (with highest number).
    pub fn last(&self) -> Option<(u64, &V)> {
        self.nums.iter().next_back().map(|(&k, v)| (k, v))
    }

    /// Get a reference to the value with the given key.
    pub fn get<'a, K>(&self, key: K) -> Option<&V>
    where
        K: Into<BorrowedKey<'a>>,
    {
        match key.into() {
            BorrowedKey::Num(num) => self.nums.get(&num),
            BorrowedKey::Str(string) => self.strs.get(string),
        }
    }

    /// Borrow the value with the given key mutably.
    pub fn get_mut<'a, K>(&mut self, key: K) -> Option<&mut V>
    where
        K: Into<BorrowedKey<'a>>,
    {
        match key.into() {
            BorrowedKey::Num(num) => self.nums.get_mut(&num),
            BorrowedKey::Str(string) => self.strs.get_mut(string),
        }
    }

    /// Insert a value into the dictionary.
    pub fn insert<K>(&mut self, key: K, value: V)
    where
        K: Into<OwnedKey>,
    {
        match key.into() {
            OwnedKey::Num(num) => {
                self.nums.insert(num, value);
                if self.lowest_free == num {
                    self.lowest_free += 1;
                }
            }
            OwnedKey::Str(string) => {
                self.strs.insert(string, value);
            }
        }
    }

    /// Remove the value with the given key from the dictionary.
    pub fn remove<'a, K>(&mut self, key: K) -> Option<V>
    where
        K: Into<BorrowedKey<'a>>,
    {
        match key.into() {
            BorrowedKey::Num(num) => {
                self.lowest_free = self.lowest_free.min(num);
                self.nums.remove(&num)
            }
            BorrowedKey::Str(string) => self.strs.remove(string),
        }
    }

    /// Append a value to the dictionary.
    ///
    /// This will associate the `value` with the lowest free number key (zero if
    /// there is no number key so far).
    pub fn push(&mut self, value: V) {
        while self.nums.contains_key(&self.lowest_free) {
            self.lowest_free += 1;
        }
        self.nums.insert(self.lowest_free, value);
        self.lowest_free += 1;
    }

    /// Iterator over all borrowed keys and values.
    pub fn iter(&self) -> impl Iterator<Item = (BorrowedKey, &V)> {
        self.nums()
            .map(|(&k, v)| (BorrowedKey::Num(k), v))
            .chain(self.strs().map(|(k, v)| (BorrowedKey::Str(k), v)))
    }

    /// Iterate over all values in the dictionary.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.nums().map(|(_, v)| v).chain(self.strs().map(|(_, v)| v))
    }

    /// Iterate over the number key-value pairs.
    pub fn nums(&self) -> std::collections::btree_map::Iter<u64, V> {
        self.nums.iter()
    }

    /// Iterate over the string key-value pairs.
    pub fn strs(&self) -> std::collections::btree_map::Iter<String, V> {
        self.strs.iter()
    }

    /// Move into an owned iterator over owned keys and values.
    pub fn into_iter(self) -> impl Iterator<Item = (OwnedKey, V)> {
        self.nums
            .into_iter()
            .map(|(k, v)| (OwnedKey::Num(k), v))
            .chain(self.strs.into_iter().map(|(k, v)| (OwnedKey::Str(k), v)))
    }

    /// Move into an owned iterator over all values in the dictionary.
    pub fn into_values(self) -> impl Iterator<Item = V> {
        self.nums
            .into_iter()
            .map(|(_, v)| v)
            .chain(self.strs.into_iter().map(|(_, v)| v))
    }

    /// Iterate over the number key-value pairs.
    pub fn into_nums(self) -> std::collections::btree_map::IntoIter<u64, V> {
        self.nums.into_iter()
    }

    /// Iterate over the string key-value pairs.
    pub fn into_strs(self) -> std::collections::btree_map::IntoIter<String, V> {
        self.strs.into_iter()
    }
}

impl<'a, K, V> Index<K> for Dict<V>
where
    K: Into<BorrowedKey<'a>>,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.get(index).expect("key not in dict")
    }
}

impl<V> Default for Dict<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Eq> Eq for Dict<V> {}

impl<V: PartialEq> PartialEq for Dict<V> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<V: Debug> Debug for Dict<V> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("()");
        }

        let mut builder = f.debug_tuple("");

        struct Entry<'a>(bool, &'a dyn Display, &'a dyn Debug);
        impl<'a> Debug for Entry<'a> {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                if self.0 {
                    f.write_str("\"")?;
                }
                self.1.fmt(f)?;
                if self.0 {
                    f.write_str("\"")?;
                }
                if f.alternate() {
                    f.write_str(" = ")?;
                } else {
                    f.write_str("=")?;
                }
                self.2.fmt(f)
            }
        }

        for (key, value) in self.nums() {
            builder.field(&Entry(false, &key, &value));
        }

        for (key, value) in self.strs() {
            builder.field(&Entry(key.contains(' '), &key, &value));
        }

        builder.finish()
    }
}

/// The owned variant of a dictionary key.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OwnedKey {
    Num(u64),
    Str(String),
}

impl From<BorrowedKey<'_>> for OwnedKey {
    fn from(key: BorrowedKey<'_>) -> Self {
        match key {
            BorrowedKey::Num(num) => Self::Num(num),
            BorrowedKey::Str(string) => Self::Str(string.to_string()),
        }
    }
}

impl From<u64> for OwnedKey {
    fn from(num: u64) -> Self {
        Self::Num(num)
    }
}

impl From<String> for OwnedKey {
    fn from(string: String) -> Self {
        Self::Str(string)
    }
}

impl From<&'static str> for OwnedKey {
    fn from(string: &'static str) -> Self {
        Self::Str(string.to_string())
    }
}

/// The borrowed variant of a dictionary key.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BorrowedKey<'a> {
    Num(u64),
    Str(&'a str),
}

impl From<u64> for BorrowedKey<'static> {
    fn from(num: u64) -> Self {
        Self::Num(num)
    }
}

impl<'a> From<&'a String> for BorrowedKey<'a> {
    fn from(string: &'a String) -> Self {
        Self::Str(&string)
    }
}

impl<'a> From<&'a str> for BorrowedKey<'a> {
    fn from(string: &'a str) -> Self {
        Self::Str(string)
    }
}

/// A dictionary entry which tracks key and value span.
#[derive(Clone, PartialEq)]
pub struct SpannedEntry<V> {
    pub key: Span,
    pub val: Spanned<V>,
}

impl<V> SpannedEntry<V> {
    /// Create a new entry.
    pub fn new(key: Span, val: Spanned<V>) -> Self {
        Self { key, val }
    }

    /// Create an entry with the same span for key and value.
    pub fn val(val: Spanned<V>) -> Self {
        Self { key: val.span, val }
    }

    /// Convert from `&SpannedEntry<T>` to `SpannedEntry<&T>`
    pub fn as_ref(&self) -> SpannedEntry<&V> {
        SpannedEntry { key: self.key, val: self.val.as_ref() }
    }

    /// Map the entry to a different value type.
    pub fn map<U>(self, f: impl FnOnce(V) -> U) -> SpannedEntry<U> {
        SpannedEntry { key: self.key, val: self.val.map(f) }
    }
}

impl<V: Debug> Debug for SpannedEntry<V> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if f.alternate() {
            f.write_str("key")?;
            self.key.fmt(f)?;
            f.write_str(" ")?;
        }
        self.val.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::Dict;

    #[test]
    fn test_dict_different_key_types_dont_interfere() {
        let mut dict = Dict::new();
        dict.insert(10, "hello");
        dict.insert("twenty", "there");
        assert_eq!(dict.len(), 2);
        assert_eq!(dict[10], "hello");
        assert_eq!(dict["twenty"], "there");
    }

    #[test]
    fn test_dict_push_skips_already_inserted_keys() {
        let mut dict = Dict::new();
        dict.insert(2, "2");
        dict.push("0");
        dict.insert(3, "3");
        dict.push("1");
        dict.push("4");
        assert_eq!(dict.len(), 5);
        assert_eq!(dict[0], "0");
        assert_eq!(dict[1], "1");
        assert_eq!(dict[2], "2");
        assert_eq!(dict[3], "3");
        assert_eq!(dict[4], "4");
    }

    #[test]
    fn test_dict_push_remove_push_reuses_index() {
        let mut dict = Dict::new();
        dict.push("0");
        dict.push("1");
        dict.push("2");
        dict.remove(1);
        dict.push("a");
        dict.push("3");
        assert_eq!(dict.len(), 4);
        assert_eq!(dict[0], "0");
        assert_eq!(dict[1], "a");
        assert_eq!(dict[2], "2");
        assert_eq!(dict[3], "3");
    }

    #[test]
    fn test_dict_first_and_last_are_correct() {
        let mut dict = Dict::new();
        assert_eq!(dict.first(), None);
        assert_eq!(dict.last(), None);
        dict.insert(4, "hi");
        dict.insert("string", "hi");
        assert_eq!(dict.first(), Some((4, &"hi")));
        assert_eq!(dict.last(), Some((4, &"hi")));
        dict.insert(2, "bye");
        assert_eq!(dict.first(), Some((2, &"bye")));
        assert_eq!(dict.last(), Some((4, &"hi")));
    }

    #[test]
    fn test_dict_format_debug() {
        let mut dict = Dict::new();
        assert_eq!(format!("{:?}", dict), "()");
        assert_eq!(format!("{:#?}", dict), "()");

        dict.insert(10, "hello");
        dict.insert("twenty", "there");
        dict.insert("sp ace", "quotes");
        assert_eq!(
            format!("{:?}", dict),
            r#"(10="hello", "sp ace"="quotes", twenty="there")"#,
        );
        assert_eq!(format!("{:#?}", dict).lines().collect::<Vec<_>>(), [
            "(",
            r#"    10 = "hello","#,
            r#"    "sp ace" = "quotes","#,
            r#"    twenty = "there","#,
            ")",
        ]);
    }
}