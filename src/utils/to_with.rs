// mochou-p/text-editor/src/utils/to_with.rs

pub trait ToWith: Ord + Copy {
    fn to_min_with(&mut self, rhs: Self) -> &mut Self {
        *self = (*self).min(rhs);
        self
    }

    fn to_max_with(&mut self, rhs: Self) -> &mut Self {
        *self = (*self).max(rhs);
        self
    }
}

impl ToWith for isize {}

