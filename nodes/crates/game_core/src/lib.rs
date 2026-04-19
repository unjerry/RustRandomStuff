//! Platform-independent gameplay state and rules.

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

