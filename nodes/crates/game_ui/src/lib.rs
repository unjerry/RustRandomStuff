//! UI layer. Add `egui` HUD, menus, debug tools, and editor panels here.

use nodes_core::GameCore;

#[derive(Debug, Default)]
pub struct UiState;

impl UiState {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _game: &GameCore) {
        // UI model sync will live here.
    }
}

