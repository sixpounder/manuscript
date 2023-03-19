use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSettings {
    paragraph_spacing: i32,
}

impl Default for DocumentSettings {
    fn default() -> Self {
        Self {
            paragraph_spacing: 36,
        }
    }
}

impl DocumentSettings {
    pub fn paragraph_spacing(&self) -> i32 {
        self.paragraph_spacing
    }

    pub fn set_paragraph_spacing(&mut self, value: i32) {
        self.paragraph_spacing = value;
    }
}
