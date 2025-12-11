// text-editor/src/preferences.rs

type Mask = u8;

pub struct PreferenceMask(pub Mask);

impl PreferenceMask {
    pub const SAVE_FILE_ON_EXIT: Mask = 0b0000_0001;

    const fn empty() -> Self {
        Self(0)
    }
}

impl Default for PreferenceMask {
    fn default() -> Self {
        let mut mask = Self::empty();

        mask.0 |= Self::SAVE_FILE_ON_EXIT;

        mask
    }
}

