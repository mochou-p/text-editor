// mochou-p/text-editor/src/insert_map.rs

//! unused since i later solved the original problem differently,
//! but gonna keep it here in case i need it in the future

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;


pub struct InsertMapIter<'a, K: Hash + Eq + Clone, V> {
    start: usize,
    end:   usize,
    inner: &'a InsertMap<K, V>
}

pub struct InsertMap<K: Hash + Eq + Clone, V> {
    indices: Vec<K>,
    map:     HashMap<K, V>
}

impl<K: Hash + Eq + Clone, V> Index<&K> for InsertMap<K, V> {
    type Output = V;

    fn index(&self, index: &K) -> &Self::Output {
        &self.map[index]
    }
}

impl<K: Hash + Eq + Clone, V> InsertMap<K, V> {
    pub fn new() -> Self {
        Self { indices: Vec::new(), map: HashMap::new() }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.map.insert(key.clone(), value).is_some() {
            self.indices.remove(
                self.indices
                    .iter()
                    .position(|k| *k == key.clone())
                    .unwrap()
            );
        }

        self.indices.push(key);
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let result = self.map.remove(key);

        if result.is_some() {
            self.indices.remove(
                self.indices
                    .iter()
                    .position(|k| k == key)
                    .unwrap()
            );
        }

        result
    }

    pub fn iter(&self) -> InsertMapIter<'_, K, V> {
        InsertMapIter { start: 0, end: self.indices.len(), inner: self }
    }
}

impl<'a, K: Hash + Eq + Clone, V> Iterator for InsertMapIter<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            self.start += 1;
            Some(&self.inner.indices[self.start - 1])
        } else {
            None
        }
    }
}

impl<'a, K: Hash + Eq + Clone, V> DoubleEndedIterator for InsertMapIter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end > self.start {
            self.end -= 1;
            Some(&self.inner.indices[self.end])
        } else {
            None
        }
    }
}
