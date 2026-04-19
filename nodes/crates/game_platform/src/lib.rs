//! Platform services. Keep platform-specific paths and APIs behind traits here.

use std::borrow::Cow;

pub trait AssetSource {
    fn read(&self, path: &str) -> Result<Cow<'_, [u8]>, AssetError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetError {
    message: String,
}

impl AssetError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

