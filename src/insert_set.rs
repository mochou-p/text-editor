// mochou-p/text-editor/src/insert_set.rs

use std::collections::HashSet;
use std::hash::Hash;


pub struct InsertSetIter<'a, K: Hash + Eq + Clone> {
    start: usize,
    inner: &'a InsertSet<K>
}

pub struct InsertSet<K: Hash + Eq + Clone> {
    indices: Vec<K>,
    set:     HashSet<K>
}

impl<K: Hash + Eq + Clone> InsertSet<K> {
    pub fn new() -> Self {
        Self { indices: Vec::new(), set: HashSet::new() }
    }

    pub fn insert(&mut self, key: K) {
        if self.set.insert(key.clone()) {
            self.indices.push(key);
        }
    }

    pub fn remove(&mut self, key: &K) -> bool {
        let result = self.set.remove(key);

        if result {
            self.indices.remove(
                self.indices
                    .iter()
                    .position(|k| k == key)
                    .unwrap()
            );
        }

        result
    }

    pub fn iter(&self) -> InsertSetIter<'_, K> {
        InsertSetIter { start: 0, inner: self }
    }
}

impl<'a, K: Hash + Eq + Clone> Iterator for InsertSetIter<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.inner.indices.len() {
            self.start += 1;
            Some(&self.inner.indices[self.start - 1])
        } else {
            None
        }
    }
}
