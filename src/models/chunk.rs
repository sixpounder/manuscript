use super::{BufferChunk, ChunkType, DocumentChunk, MutableBufferChunk};
use crate::{
    models::prelude::{Color, ManuscriptResult},
    services::i18n::i18n,
};
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

    fn set_priority(&mut self, _value: Option<u64>) {}

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
    include_in_compilation: bool,
    accent: Color,
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
            Some(title) => {
                if title.is_empty() {
                    None
                } else {
                    Some(title.as_str())
                }
            }
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

    fn include_in_compilation(&self) -> bool {
        self.include_in_compilation
    }

    fn set_include_in_compilation(&mut self, value: bool) -> ManuscriptResult<()> {
        self.include_in_compilation = value;
        Ok(())
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
            include_in_compilation: true,
            accent: Color::default(),
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
    include_in_compilation: bool,
    accent: Color,
    locked: bool,
    name: Option<String>,
    gender: Gender,
    age: Option<u32>,
    role: Option<String>,
    physical_traits: Bytes,
    psycological_traits: Bytes,
    background: Bytes,
}

impl Default for CharacterSheet {
    fn default() -> Self {
        Self {
            id: Ulid::new().into(),
            priority: 0,
            include_in_compilation: true,
            accent: Color::default(),
            locked: false,
            name: None,
            gender: Gender::Unspecified,
            age: None,
            role: None,
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

    pub fn set_name(&mut self, value: Option<String>) {
        self.name = value;
    }

    pub fn role(&self) -> Option<&String> {
        self.role.as_ref()
    }

    pub fn set_role(&mut self, value: Option<String>) {
        self.role = value;
    }

    pub fn gender(&self) -> Gender {
        self.gender
    }

    pub fn set_gender(&mut self, value: Gender) {
        self.gender = value;
    }

    pub fn age(&self) -> Option<u32> {
        self.age
    }

    pub fn set_age(&mut self, value: Option<u32>) {
        self.age = value;
    }

    pub fn background(&self) -> &[u8] {
        self.background.as_ref()
    }

    pub fn set_background(&mut self, value: &[u8]) {
        self.background = Bytes::from(Vec::from(value));
    }

    pub fn set_background_bytes(&mut self, value: Bytes) {
        self.background = value;
    }

    pub fn physical_traits(&self) -> &[u8] {
        self.physical_traits.as_ref()
    }

    pub fn set_physical_traits(&mut self, value: &[u8]) {
        self.physical_traits = Bytes::from(Vec::from(value))
    }

    pub fn set_physical_traits_bytes(&mut self, value: Bytes) {
        self.physical_traits = value;
    }

    pub fn psycological_traits(&self) -> &[u8] {
        self.physical_traits.as_ref()
    }

    pub fn set_psycological_traits(&mut self, value: &[u8]) {
        self.psycological_traits = Bytes::from(Vec::from(value))
    }

    pub fn set_psycological_traits_bytes(&mut self, value: Bytes) {
        self.psycological_traits = value;
    }
}

impl DocumentChunk for CharacterSheet {
    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn title(&self) -> Option<&str> {
        match self.name.as_ref() {
            Some(title) => {
                if title.is_empty() {
                    None
                } else {
                    Some(title.as_str())
                }
            }
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

    fn include_in_compilation(&self) -> bool {
        self.include_in_compilation
    }

    fn set_include_in_compilation(&mut self, value: bool) -> ManuscriptResult<()> {
        self.include_in_compilation = value;
        Ok(())
    }

    fn accent(&self) -> Color {
        self.accent.clone()
    }

    fn set_accent(&mut self, value: Color) -> ManuscriptResult<()> {
        self.accent = value;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Other,
    Unspecified,
}

impl From<u64> for Gender {
    fn from(idx: u64) -> Self {
        match idx {
            0 => Self::Male,
            1 => Self::Female,
            2 => Self::Other,
            _ => Self::Unspecified,
        }
    }
}

impl From<u32> for Gender {
    fn from(idx: u32) -> Self {
        Self::from(idx as u64)
    }
}

impl From<Gender> for u64 {
    fn from(source: Gender) -> Self {
        match source {
            Gender::Male => 0,
            Gender::Female => 1,
            Gender::Other => 2,
            Gender::Unspecified => 10,
        }
    }
}

impl From<Gender> for u32 {
    fn from(source: Gender) -> Self {
        match source {
            Gender::Male => 0,
            Gender::Female => 1,
            Gender::Other => 2,
            Gender::Unspecified => 10,
        }
    }
}

impl Default for Gender {
    fn default() -> Self {
        Self::Unspecified
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
