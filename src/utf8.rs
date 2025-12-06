// text-editor/src/utf8.rs

pub trait Utf8Len {
    fn utf8_len(&self) -> usize;
}

impl Utf8Len for str {
    fn utf8_len(&self) -> usize {
        self.chars().count()
    }
}

impl Utf8Len for String {
    fn utf8_len(&self) -> usize {
        self.as_str().utf8_len()
    }
}

pub trait Utf8Range {
    fn utf8_range(&self, start: usize, end: usize) -> String;
}

impl Utf8Range for str {
    fn utf8_range(&self, start: usize, end: usize) -> String {
        self.chars()
            .skip(start)
            .take(end - start)
            .collect()
    }
}

impl Utf8Range for String {
    fn utf8_range(&self, start: usize, end: usize) -> String {
        self.as_str().utf8_range(start, end)
    }
}

pub trait Utf8Index {
    fn utf8_index(&self, idx: usize) -> usize;
}

impl Utf8Index for str {
    fn utf8_index(&self, idx: usize) -> usize {
        self.char_indices()
            .nth(idx)
            .map_or_else(|| self.len(), |(i, _)| i)
    }
}

impl Utf8Index for String {
    fn utf8_index(&self, idx: usize) -> usize {
        self.as_str().utf8_index(idx)
    }
}

pub trait Utf8Insert {
    fn utf8_insert(&mut self, idx: usize, ch: char);
}

impl Utf8Insert for String {
    fn utf8_insert(&mut self, idx: usize, ch: char) {
        self.insert(self.utf8_index(idx), ch);
    }
}

pub trait Utf8Remove {
    fn utf8_remove(&mut self, idx: usize);
}

impl Utf8Remove for String {
    fn utf8_remove(&mut self, idx: usize) {
        self.remove(self.utf8_index(idx));
    }
}

pub trait Utf8SplitOff {
    fn utf8_split_off(&mut self, at: usize) -> String;
}

impl Utf8SplitOff for String {
    fn utf8_split_off(&mut self, at: usize) -> String {
        self.split_off(self.utf8_index(at))
    }
}

pub trait Utf8Drain {
    fn utf8_drain(&mut self, start: usize, end: usize);
}

impl Utf8Drain for String {
    fn utf8_drain(&mut self, start: usize, end: usize) {
        self.drain(
            self.utf8_index(start)
            ..
            self.utf8_index(end)
        );
    }
}

