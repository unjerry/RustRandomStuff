//! Platform-independent gameplay state and rules.

mod node_template;

pub use node_template::{
    Color, NodeSize, NodeStyle, NodeTemplate, PortDirection, PortKind, PortTemplate,
    PropertyTemplate, TemplateError, TemplateId, TemplateValue,
};

#[derive(Debug, Default)]
pub struct GameCore {
    tick: u64,
}

impl GameCore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) {
        self.tick += 1;
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }
}
