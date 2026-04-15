// mochou-p/text-editor/src/utf8.rs

type Index  = isize;
type Length = isize;

#[allow(dead_code)]
pub trait Utf8 {
    fn utf8_len  (&self                          ) -> Length;
    fn utf8_range(&self, start: Index, end: Index) -> String;
    fn utf8_index(&self, idx:   Index            ) ->  Index;
}

#[allow(dead_code)]
pub trait Utf8Mut {
    fn utf8_insert    (&mut self, idx:   Index, ch:  char )          ;
    fn utf8_insert_str(&mut self, idx:   Index, ch:  &str )          ;
    fn utf8_remove    (&mut self, idx:   Index            )          ;
    fn utf8_split_off (&mut self, at:    Index            ) -> String;
    fn utf8_drain     (&mut self, start: Index, end: Index)          ;
}

impl Utf8 for str {
    fn utf8_len(&self) -> Length {
        self.chars().count() as Length
    }

    fn utf8_range(&self, start: isize, end: isize) -> String {
        self.chars()
            .skip(start as usize)
            .take((end - start) as usize)
            .collect()
    }

    fn utf8_index(&self, idx: isize) -> Index {
        self.char_indices()
            .nth(idx as usize)
            // NOTE: wait why did i map else to len
            .map_or_else(|| self.len(), |(i, _)| i)
            as Index
    }
}

impl Utf8 for String {
    fn utf8_len(&self) -> Length {
        self.as_str().utf8_len()
    }

    fn utf8_range(&self, start: isize, end: isize) -> String {
        self.as_str().utf8_range(start, end)
    }

    fn utf8_index(&self, idx: isize) -> Index {
        self.as_str().utf8_index(idx)
    }
}

impl Utf8Mut for String {
    fn utf8_insert(&mut self, idx: Index, ch: char) {
        self.insert(self.utf8_index(idx) as usize, ch);
    }

    fn utf8_insert_str(&mut self, idx: Index, s: &str) {
        self.insert_str(self.utf8_index(idx) as usize, s);
    }

    fn utf8_remove(&mut self, idx: Index) {
        self.remove(self.utf8_index(idx) as usize);
    }

    fn utf8_split_off(&mut self, at: Index) -> String {
        self.split_off(self.utf8_index(at) as usize)
    }

    fn utf8_drain(&mut self, start: Index, end: Index) {
        self.drain(
            self.utf8_index(start) as usize
            ..
            self.utf8_index(end) as usize
        );
    }
}

