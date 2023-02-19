use super::{Chapter, CharacterSheet};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Chapter,
    CharacterSheet,
}

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum ManuscriptError {
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
    fn chunk_type(&self) -> ChunkType;
    fn priority(&self) -> Option<u64>;
    fn set_priority(&mut self, value: Option<u64>);
    fn locked(&self) -> bool;
    fn set_locked(&mut self, value: bool);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
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
}

pub trait MutableBufferChunk: BufferChunk {
    fn set_buffer(&mut self, value: Bytes);
}
