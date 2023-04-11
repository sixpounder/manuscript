use serde::{Deserialize, Serialize};

#[derive(glib::ValueDelegate, Debug, Clone, Copy, Serialize, Deserialize)]
#[value_delegate(from = u8)]
#[repr(u8)]
pub enum TextMetricSize {
    Narrow,
    Medium,
    Wide,
}

impl From<TextMetricSize> for u8 {
    fn from(value: TextMetricSize) -> Self {
        match value {
            TextMetricSize::Narrow => 0,
            TextMetricSize::Medium => 1,
            TextMetricSize::Wide => 2,
        }
    }
}

impl From<&TextMetricSize> for u8 {
    fn from(value: &TextMetricSize) -> Self {
        u8::from(*value)
    }
}

impl From<u8> for TextMetricSize {
    fn from(value: u8) -> Self {
        match value {
            0 => TextMetricSize::Narrow,
            1 => TextMetricSize::Medium,
            2 => TextMetricSize::Wide,
            _ => unreachable!("Not a valid metric size value"),
        }
    }
}

impl From<&str> for TextMetricSize {
    fn from(value: &str) -> Self {
        match value {
            "Narrow" => Self::Narrow,
            "Medium" => Self::Medium,
            "Wide" => Self::Wide,
            _ => unreachable!(),
        }
    }
}

impl From<TextMetricSize> for &str {
    fn from(value: TextMetricSize) -> Self {
        match value {
            TextMetricSize::Narrow => "Narrow",
            TextMetricSize::Medium => "Medium",
            TextMetricSize::Wide => "Wide",
        }
    }
}

impl From<f64> for TextMetricSize {
    fn from(value: f64) -> Self {
        Self::from(value.abs() as u8)
    }
}

impl Default for TextMetricSize {
    fn default() -> Self {
        Self::Narrow
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSettings {
    line_height: TextMetricSize,
    paragraph_spacing: TextMetricSize,
}

impl DocumentSettings {
    pub fn paragraph_spacing(&self) -> TextMetricSize {
        self.paragraph_spacing
    }

    pub fn set_paragraph_spacing(&mut self, value: TextMetricSize) {
        self.paragraph_spacing = value;
    }

    pub fn line_height(&self) -> TextMetricSize {
        self.line_height
    }

    pub fn set_line_height(&mut self, value: TextMetricSize) {
        self.line_height = value;
    }
}
