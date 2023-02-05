use super::{BufferChunk, ChunkType, DocumentChunk, MutableBufferChunk};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DocumentManifest {
    title: Option<String>,
    author: String,
}

impl Default for DocumentManifest {
    fn default() -> Self {
        Self {
            title: None,
            author: gtk::glib::real_name()
                .into_string()
                .unwrap_or(String::from("")),
        }
    }
}

/// A Chapter is a chunk representing the content of a single manuscript chapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    id: String,
    priority: u64,
    title: String,
    buffer: Bytes,
    notes: Vec<Note>,
}

impl DocumentChunk for Chapter {
    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn chunk_type(&self) -> ChunkType {
        ChunkType::Chapter
    }

    fn priority(&self) -> Option<u64> {
        Some(self.priority)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BufferChunk for Chapter {
    fn buffer(&self) -> &Bytes {
        &self.buffer
    }
}

impl MutableBufferChunk for Chapter {
    fn set_buffer(&mut self, value: Bytes) {
        self.buffer = value;
    }
}

impl Default for Chapter {
    fn default() -> Self {
        Self {
            id: Ulid::new().into(),
            priority: 0,
            title: String::from(""),
            buffer: Bytes::from(""),
            notes: vec![],
        }
    }
}

impl Chapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn set_title(&mut self, value: &str) {
        self.title = String::from(value)
    }

    pub fn add_note(&mut self, from: NoteOffsetType, to: NoteOffsetType, content: String) {
        self.notes.push(Note {
            id: Ulid::new().into(),
            buffer: Bytes::from(content),
            offset: from,
            len: to - from,
        });
    }

    pub fn notes_at(&self, offset: NoteOffsetType) -> Vec<&Note> {
        self.notes
            .iter()
            .filter(|n| n.offset_start() <= offset && n.offset_end() >= offset)
            .collect::<Vec<&Note>>()
    }

    pub fn note_at_mut(&mut self, offset: NoteOffsetType) -> Vec<&mut Note> {
        self.notes
            .iter_mut()
            .filter(|n| n.offset_start() <= offset && n.offset_end() >= offset)
            .collect::<Vec<&mut Note>>()
    }
}

/// A CharacterSheet is a chunk representing the description of a character
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct CharacterSheet {
    id: String,
    priority: u64,
    name: String,
    physical_traits: Bytes,
    psycological_traits: Bytes,
    background: Bytes,
}

impl DocumentChunk for CharacterSheet {
    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn chunk_type(&self) -> ChunkType {
        ChunkType::CharacterSheet
    }

    fn priority(&self) -> Option<u64> {
        Some(self.priority)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

type NoteOffsetType = i32;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Note {
    id: String,
    buffer: Bytes,
    offset: NoteOffsetType,
    len: NoteOffsetType,
}

impl Note {
    pub fn offset_start(&self) -> NoteOffsetType {
        self.offset
    }

    pub fn offset_end(&self) -> NoteOffsetType {
        self.offset + self.len
    }

    pub fn offset_len(&self) -> NoteOffsetType {
        self.len
    }

    pub fn buffer(&self) -> &[u8] {
        self.buffer.as_ref()
    }

    pub fn set_buffer(&mut self, value: Bytes) {
        self.buffer = value;
    }
}
