// text-editor/src/to.rs

pub trait ToMinWith: Ord + Copy {
    fn to_min_with(&mut self, rhs: Self) -> &mut Self {
        *self = (*self).min(rhs);
        self
    }
}

impl ToMinWith for usize {}

pub trait ToMaxWith: Ord + Copy {
    fn to_max_with(&mut self, rhs: Self) -> &mut Self {
        *self = (*self).max(rhs);
        self
    }
}

impl ToMaxWith for usize {}

