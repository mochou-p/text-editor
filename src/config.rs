// text-editor/src/config.rs

use termion::event::Event;

use super::actions::ActionIndex;
use super::keybinds::Keybinds;
use super::preferences::PreferenceMask;


pub struct Config {
        keybinds:    Keybinds,
    pub preferences: PreferenceMask
}

impl Default for Config {
    fn default() -> Self {
        let keybinds    = Keybinds::default();
        let preferences = PreferenceMask::default();

        Self { keybinds, preferences }
    }
}

impl Config {
    pub fn get_bound_action_index(&self, event: &Event) -> Option<&ActionIndex> {
        self.keybinds.0.get(event)
    }
}

