use super::{BufferChunk, ChunkType, DocumentChunk, MutableBufferChunk};
use crate::services::i18n::i18n;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DocumentManifest {
    title: Option<String>,
    author: String,
    locked: bool,
}

impl Default for DocumentManifest {
    fn default() -> Self {
        Self {
            title: None,
            author: gtk::glib::real_name()
                .into_string()
                .unwrap_or(String::from("")),
            locked: false,
        }
    }
}

impl DocumentManifest {
    pub fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }
    pub fn set_title(&mut self, value: Option<String>) {
        self.title = value;
    }
}

impl DocumentChunk for DocumentManifest {
    fn id(&self) -> &str {
        "manifest"
    }

    fn title(&self) -> Option<&str> {
        match self.title.as_ref() {
            Some(title) => Some(title.as_str()),
            None => None,
        }
    }

    fn default_title(&self) -> &str {
        "Untitled manuscript"
    }

    fn chunk_type(&self) -> ChunkType {
        ChunkType::Manifest
    }

    fn category_name(&self) -> String {
        i18n("Manifest")
    }

    fn priority(&self) -> Option<u64> {
        Some(100)
    }

    fn set_priority(&mut self, value: Option<u64>) {}

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn locked(&self) -> bool {
        self.locked
    }

    fn set_locked(&mut self, value: bool) {
        self.locked = value;
    }
}

/// A Chapter is a chunk representing the content of a single manuscript chapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    id: String,
    priority: u64,
    locked: bool,
    title: Option<String>,
    buffer: Bytes,
    notes: Vec<Note>,
}

impl DocumentChunk for Chapter {
    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn title(&self) -> Option<&str> {
        match self.title.as_ref() {
            Some(title) => Some(title.as_str()),
            None => None,
        }
    }

    fn default_title(&self) -> &str {
        "Untitled chapter"
    }

    fn chunk_type(&self) -> ChunkType {
        ChunkType::Chapter
    }

    fn category_name(&self) -> String {
        i18n("Chapters")
    }

    fn priority(&self) -> Option<u64> {
        Some(self.priority)
    }

    fn set_priority(&mut self, value: Option<u64>) {
        self.priority = value.unwrap_or(0);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn locked(&self) -> bool {
        self.locked
    }

    fn set_locked(&mut self, value: bool) {
        self.locked = value;
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
            locked: false,
            title: None,
            buffer: Bytes::from(""),
            notes: vec![],
        }
    }
}

impl Chapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    pub fn set_title(&mut self, value: &str) {
        self.title = Some(String::from(value))
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CharacterSheet {
    id: String,
    priority: u64,
    locked: bool,
    name: Option<String>,
    synopsis: Option<String>,
    physical_traits: Bytes,
    psycological_traits: Bytes,
    background: Bytes,
}

impl Default for CharacterSheet {
    fn default() -> Self {
        Self {
            id: Ulid::new().into(),
            priority: 0,
            locked: false,
            name: None,
            synopsis: None,
            physical_traits: Bytes::new(),
            psycological_traits: Bytes::new(),
            background: Bytes::new(),
        }
    }
}

impl CharacterSheet {
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn synopsis(&self) -> Option<&String> {
        self.synopsis.as_ref()
    }
}

impl DocumentChunk for CharacterSheet {
    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn title(&self) -> Option<&str> {
        match self.name.as_ref() {
            Some(title) => Some(title.as_str()),
            None => None,
        }
    }

    fn default_title(&self) -> &str {
        "Unnamed character"
    }

    fn chunk_type(&self) -> ChunkType {
        ChunkType::CharacterSheet
    }

    fn category_name(&self) -> String {
        i18n("Character sheets")
    }

    fn priority(&self) -> Option<u64> {
        Some(self.priority)
    }

    fn set_priority(&mut self, value: Option<u64>) {
        self.priority = value.unwrap_or(0);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn locked(&self) -> bool {
        self.locked
    }

    fn set_locked(&mut self, value: bool) {
        self.locked = value;
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
