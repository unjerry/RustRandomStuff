//! UI layer. Add `egui` HUD, menus, debug tools, and editor panels here.

use nodes_core::GameCore;

mod node_view;

pub use node_view::{
    show_node_editor, NodeGraphEditor, NodeInstanceId, NodeRenderPlan, Point, PortRenderPlan,
    PropertyRenderPlan, Size, TextAlign,
};

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
