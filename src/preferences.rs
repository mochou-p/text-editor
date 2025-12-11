// text-editor/src/preferences.rs

type Mask = u16;

pub struct PreferenceMask(pub Mask);

// TODO: macro
impl PreferenceMask {
    pub const FILE_SAVE_ON_EDITOR_EXIT:                            Mask = 0b0000_0000_0000_0001;

    pub const CURSOR_MOVE_LEFT_WRAPS_AT_LINE_BOUNDARY:             Mask = 0b0000_0000_0000_0010;
    pub const CURSOR_MOVE_RIGHT_WRAPS_AT_LINE_BOUNDARY:            Mask = 0b0000_0000_0000_0100;
    pub const CURSOR_MOVE_TO_PREVIOUS_WORD_WRAPS_AT_LINE_BOUNDARY: Mask = 0b0000_0000_0000_1000;
    pub const CURSOR_MOVE_TO_NEXT_WORD_WRAPS_AT_LINE_BOUNDARY:     Mask = 0b0000_0000_0001_0000;
    pub const CURSOR_MOVE_UP_SCROLLS_AT_SCREEN_BOUNDARY:           Mask = 0b0000_0000_0010_0000;
    pub const CURSOR_MOVE_DOWN_SCROLLS_AT_SCREEN_BOUNDARY:         Mask = 0b0000_0000_0100_0000;
    pub const CURSOR_MOVE_UP_GOES_TO_START_OF_FILE_AT_FIRST_LINE:  Mask = 0b0000_0000_1000_0000;
    pub const CURSOR_MOVE_DOWN_GOES_TO_END_OF_FILE_AT_LAST_LINE:   Mask = 0b0000_0001_0000_0000;

    pub const TYPING_ERASE_CHARACTER_LEFT_WRAPS_AT_LINE_BOUNDARY:  Mask = 0b0000_0010_0000_0000;
    pub const TYPING_ERASE_CHARACTER_RIGHT_WRAPS_AT_LINE_BOUNDARY: Mask = 0b0000_0100_0000_0000;
    pub const TYPING_ERASE_WORD_LEFT_WRAPS_AT_LINE_BOUNDARY:       Mask = 0b0000_1000_0000_0000;
    pub const TYPING_ERASE_WORD_RIGHT_WRAPS_AT_LINE_BOUNDARY:      Mask = 0b0001_0000_0000_0000;

    const fn all() -> Self {
        Self(Mask::MAX)
    }
}

impl Default for PreferenceMask {
    fn default() -> Self {
        Self(
            Self::all().0
                & !(Self::FILE_SAVE_ON_EDITOR_EXIT)
        )
    }
}

