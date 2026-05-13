// mochou-p/text-editor/src/ivec2.rs

#[derive(Clone, Copy)]
pub struct Ivec2 {
    pub x: isize,
    pub y: isize
}

impl std::ops::Add for Ivec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl std::ops::Sub for Ivec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl From<(u16, u16)> for Ivec2 {
    fn from(value: (u16, u16)) -> Self {
        Self { x: value.0 as isize, y: value.1 as isize }
    }
}

impl Default for Ivec2 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Ivec2 {
    pub const ZERO: Self = Self { x: 0, y: 0 };
    pub const ONE:  Self = Self { x: 1, y: 1 };

    pub fn new(x: u16, y: u16) -> Self {
        Self { x: x as isize, y: y as isize }
    }
}
