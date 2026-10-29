//! An insertion-ordered map used for static game data.

use std::borrow::Borrow;
use std::collections::BTreeMap;

/// A read-only map that keeps the order of its source file and supports lookup by code.
///
/// Ruby hashes keep insertion order and the original app relies on it for display
/// order and for "first match" searches, so a plain `BTreeMap` isn't enough.
#[derive(Debug, Clone)]
pub struct CodeMap<K, V> {
    entries: Vec<(K, V)>,
    index: BTreeMap<K, usize>,
}

impl<K: Ord + Clone, V> CodeMap<K, V> {
    /// Builds a map from entries in source order. A repeated key keeps its last value.
    ///
    /// # Arguments
    /// * `entries` - key/value pairs in the order they should be iterated.
    ///
    /// # Returns
    /// The map.
    pub fn new(entries: Vec<(K, V)>) -> Self {
        let index = entries
            .iter()
            .enumerate()
            .map(|(position, (key, _))| (key.clone(), position))
            .collect();
        Self { entries, index }
    }

    /// Looks up a value by key.
    ///
    /// # Returns
    /// The value, or `None` when the key is unknown.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.index
            .get(key)
            .and_then(|&position| self.entries.get(position))
            .map(|(_, value)| value)
    }

    /// Tells whether the key exists.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.index.contains_key(key)
    }

    /// Iterates over the entries in source order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter().map(|(key, value)| (key, value))
    }

    /// Iterates over the keys in source order.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|(key, _)| key)
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Tells whether the map has no entry.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
