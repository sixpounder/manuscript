use bytes::Bytes;
use gtk::prelude::TextBufferExt;
use serde::{Deserialize, Serialize};
use super::{Chapter, CharacterSheet};

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Chapter,
    CharacterSheet,
}

#[derive(Debug, Clone)]
pub enum ManuscriptError {
    DocumentSerialize,
    DocumentDeserialize,
    ChunkParse,
    ChunkBusy,
    ChunkUnavailable,
}

pub trait DocumentChunk {
    fn id(&self) -> &str;
    fn chunk_type(&self) -> ChunkType;
    fn priority(&self) -> Option<u64>;
    fn as_any(&self) -> &dyn std::any::Any;
}

impl dyn DocumentChunk {
    pub fn new_of_type<C>() -> C
    where
        C: DocumentChunk + Default,
    {
        C::default()
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
}

impl From<&dyn BufferChunk> for gtk::TextBuffer {
    fn from(source: &dyn BufferChunk) -> Self {
        bytes_to_text_buffer(source.buffer().slice(..))
    }
}

impl From<Box<dyn BufferChunk>> for gtk::TextBuffer {
    fn from(source: Box<dyn BufferChunk>) -> Self {
        bytes_to_text_buffer(source.buffer().slice(..))
    }
}

pub trait MutableBufferChunk: BufferChunk {
    fn set_buffer(&mut self, value: Bytes);
}

impl From<&dyn MutableBufferChunk> for gtk::TextBuffer {
    fn from(source: &dyn MutableBufferChunk) -> Self {
        bytes_to_text_buffer(source.buffer().slice(..))
    }
}

impl From<Box<dyn MutableBufferChunk>> for gtk::TextBuffer {
    fn from(source: Box<dyn MutableBufferChunk>) -> Self {
        bytes_to_text_buffer(source.buffer().slice(..))
    }
}

impl From<Box<&dyn MutableBufferChunk>> for gtk::TextBuffer {
    fn from(source: Box<&dyn MutableBufferChunk>) -> Self {
        bytes_to_text_buffer(source.buffer().slice(..))
    }
}

pub fn bytes_to_text_buffer(source: Bytes) -> gtk::TextBuffer {
    let text_buffer = gtk::TextBuffer::new(None);
    text_buffer.set_text(String::from_utf8(source.to_vec()).unwrap().as_str());
    text_buffer
}
