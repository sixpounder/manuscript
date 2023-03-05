use super::{Chapter, CharacterSheet};
use bytes::Bytes;
use glib::{StaticType, Type};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

pub enum Color {
    Default,
    Cyan,
    Yellow,
    Green,
    Custom(u8, u8, u8),
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Manifest,
    Chapter,
    CharacterSheet,
}

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

    fn accent(&self) -> Color {
        Color::Default
    }

    fn set_accent(&mut self, _value: Color) {}

    fn safe_title(&self) -> &str {
        self.title().unwrap_or(self.default_title())
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

pub trait BufferChunk {
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
            if c == b' ' || c == b'\n' || c == b'\t' || c == b'\r' {
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
