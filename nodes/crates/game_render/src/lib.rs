//! Rendering layer. Add `wgpu` setup here once the game direction is known.

use nodes_core::GameCore;

#[derive(Debug, Default)]
pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Self
    }

    pub fn draw_frame(&mut self, _game: &GameCore) {
        // WGPU frame encoding will live here.
    }
}
