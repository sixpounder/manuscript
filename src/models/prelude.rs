use super::{Chapter, CharacterSheet};
use bytes::Bytes;
use glib::{StaticType, Type};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

const DEFAULT_CONTRAST_LIGHT: Color = Color(0.0, 0.0, 0.0, 0.9);
const DEFAULT_CONTRAST_DARK: Color = Color(250.0, 250.0, 250.0, 0.9);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color(pub f32, pub f32, pub f32, pub f32);

impl Color {
    pub fn is_dark(&self) -> bool {
        !self.is_light()
    }

    // Calculate the perceived brightness of the color using the formula
    // (0.299 * R + 0.587 * G + 0.114 * B) * A
    pub fn is_light(&self) -> bool {
        let brightness = (0.299 * self.0 + 0.587 * self.1 + 0.114 * self.2) * self.3;
        // If the brightness is greater than or equal to 0.6, the color is considered light
        brightness >= 0.6
    }

    pub fn contrast_color(&self) -> Color {
        if self.is_light() {
            DEFAULT_CONTRAST_LIGHT
        } else {
            DEFAULT_CONTRAST_DARK
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rgba({}, {}, {}, {})",
            self.0.ceil(),
            self.1.ceil(),
            self.2.ceil(),
            self.3
        )
    }
}

impl From<gtk::gdk::RGBA> for Color {
    fn from(value: gtk::gdk::RGBA) -> Self {
        Self(
            value.red() * 255.0,
            value.green() * 255.0,
            value.blue() * 255.0,
            value.alpha(),
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Manifest,
    Chapter,
    CharacterSheet,
}

// impl ToString for ChunkType {
//     fn to_string(&self) -> String {
//         match self {
//             ChunkType::Manifest => "Manifest".into(),
//             ChunkType::Chapter => "Chapter".into(),
//             ChunkType::CharacterSheet => "Character Sheet".into(),
//         }
//     }
// }

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = match self {
            ChunkType::Manifest => "Manifest",
            ChunkType::Chapter => "Chapter",
            ChunkType::CharacterSheet => "Character Sheet",
        };
        write!(f, "{desc}")
    }
}

#[derive(Debug, Clone)]
pub enum ManuscriptError {
    NoDocument,
    NoBackend,
    Save,
    Open(String),
    DocumentLock,
    DocumentSerialize,
    DocumentDeserialize,
    ChunkParse,
    ChunkBusy,
    ChunkUnavailable,
    Reason(&'static str),
}

pub type ManuscriptResult<T> = Result<T, ManuscriptError>;

pub trait DocumentChunk {
    fn id(&self) -> &str;
    fn title(&self) -> Option<&str>;
    fn default_title(&self) -> &str;
    fn chunk_type(&self) -> ChunkType;
    fn category_name(&self) -> String;
    fn priority(&self) -> Option<u64>;
    fn set_priority(&mut self, value: Option<u64>);
    fn locked(&self) -> bool;
    fn set_locked(&mut self, value: bool);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn accent(&self) -> Option<Color> {
        None
    }

    fn set_accent(&mut self, _value: Option<Color>) -> ManuscriptResult<()> {
        Err(ManuscriptError::Reason("Not implemented"))
    }

    fn safe_title(&self) -> &str {
        self.title().unwrap_or(self.default_title())
    }

    fn include_in_compilation(&self) -> bool {
        true
    }

    fn set_include_in_compilation(&mut self, _value: bool) -> ManuscriptResult<()> {
        Err(ManuscriptError::Reason("Not implemented"))
    }
}

impl StaticType for dyn DocumentChunk {
    fn static_type() -> Type {
        Type::OBJECT
    }
}

impl std::fmt::Debug for dyn DocumentChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.chunk_type(), self.id())
    }
}

impl dyn DocumentChunk {
    pub fn new_of_type<C>() -> C
    where
        C: DocumentChunk + Default,
    {
        C::default()
    }
}

impl<C> DocumentChunk for Box<C>
where
    C: DocumentChunk,
{
    fn id(&self) -> &str {
        self.deref().id()
    }

    fn title(&self) -> Option<&str> {
        self.deref().title()
    }

    fn default_title(&self) -> &str {
        self.deref().default_title()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self.deref().as_any()
    }

    fn priority(&self) -> Option<u64> {
        self.deref().priority()
    }

    fn set_priority(&mut self, value: Option<u64>) {
        self.deref_mut().set_priority(value);
    }

    fn chunk_type(&self) -> ChunkType {
        self.deref().chunk_type()
    }

    fn category_name(&self) -> String {
        self.deref().category_name()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self.deref_mut().as_any_mut()
    }

    fn locked(&self) -> bool {
        self.deref().locked()
    }

    fn set_locked(&mut self, value: bool) {
        self.deref_mut().set_locked(value);
    }
}

impl Clone for Box<dyn DocumentChunk> {
    fn clone(&self) -> Self {
        let any = self.as_any();
        if let Some(chapter) = any.downcast_ref::<Chapter>() {
            Box::new(chapter.clone())
        } else if let Some(character_sheet) = any.downcast_ref::<CharacterSheet>() {
            Box::new(character_sheet.clone())
        } else {
            unreachable!()
        }
    }
}

pub trait SerializableChunk<'de>: DocumentChunk + Serialize + Deserialize<'de> {}

pub trait BufferChunk: DocumentChunk {
    fn buffer(&self) -> &Bytes;

    /// Counts the words in the buffer
    fn words_count(&self) -> u64 {
        self.buffer().words_count()
    }

    /// Estimates the reading time of this buffer
    fn estimate_reading_time(&self) -> (u64, u64) {
        self.buffer().estimate_reading_time()
    }
}

pub trait MutableBufferChunk: BufferChunk {
    fn set_buffer(&mut self, value: Bytes);
}

pub trait BufferAnalytics {
    /// Counts the words in the buffer
    fn words_count(&self) -> u64;

    /// Estimates the reading time of this buffer
    fn estimate_reading_time(&self) -> (u64, u64);
}

impl BufferAnalytics for Bytes {
    /// Counts the words in the buffer
    fn words_count(&self) -> u64 {
        let mut state = 0;
        let mut wc: u64 = 0;
        let buffer = self;
        for i in 0..buffer.len() {
            let c = buffer[i];
            if c == b' ' || c == b'\n' || c == b'\t' || c == b'\r' || c == b'#' {
                state = 0;
            } else if state == 0 {
                state = 1;
                wc += 1;
            }
        }

        wc
    }

    /// Estimates the reading time of this buffer
    fn estimate_reading_time(&self) -> (u64, u64) {
        let words = self.words_count();
        let words_divided_two_hundreds = words as f64 / 200.0;
        let minutes = words_divided_two_hundreds.floor() as u64;
        let seconds = (words_divided_two_hundreds * 0.60) as u64;

        (minutes, seconds)
    }
}
