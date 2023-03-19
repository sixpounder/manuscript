use super::{
    chunk::{Chapter, CharacterSheet, DocumentManifest},
    prelude::*,
    DocumentSettings,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Document {
    manifest: DocumentManifest,
    settings: DocumentSettings,
    chunks: HashMap<String, Box<dyn DocumentChunk>>,
}

impl glib::StaticType for Document {
    fn static_type() -> glib::Type {
        glib::Type::OBJECT
    }
}

impl Document {
    pub fn title(&self) -> Option<&String> {
        self.manifest.title()
    }

    pub fn set_title(&mut self, value: Option<String>) {
        self.manifest.set_title(value);
    }

    pub fn settings(&self) -> &DocumentSettings {
        &self.settings
    }

    pub fn add_chunk<C: DocumentChunk + 'static>(&mut self, value: C) {
        self.chunks
            .insert(String::from(value.id()), Box::new(value));
    }

    pub fn remove_chunk(&mut self, id: &String) -> Option<Box<dyn DocumentChunk>> {
        self.chunks.remove(id)
    }

    pub fn get_chunk_ref(&self, id: &str) -> Option<&dyn DocumentChunk> {
        match self.chunks.get(id) {
            Some(chunk) => Some(chunk.as_ref()),
            None => None,
        }
    }

    pub fn get_chunk_mut(&mut self, id: &str) -> Option<&mut dyn DocumentChunk> {
        if let Some(chunk) = self.chunks.get_mut(id) {
            Some(chunk.as_mut())
        } else {
            None
        }
    }

    pub fn manifest(&self) -> &DocumentManifest {
        &self.manifest
    }

    pub fn chunks(&self) -> Vec<&dyn DocumentChunk> {
        self.chunks
            .values()
            .map(|c| c.as_ref())
            .collect::<Vec<&dyn DocumentChunk>>()
    }

    pub fn chunks_by_type_ref(&self, ty: ChunkType) -> Vec<&dyn DocumentChunk> {
        self.chunks
            .values()
            .filter(|v| v.chunk_type() == ty)
            .map(|c| c.as_ref())
            .collect::<Vec<&dyn DocumentChunk>>()
    }

    pub fn chunks_by_type(&mut self, ty: ChunkType) -> Vec<&dyn DocumentChunk> {
        self.chunks
            .values()
            .filter(|v| v.chunk_type() == ty)
            .map(|c| c.as_ref())
            .collect::<Vec<&dyn DocumentChunk>>()
    }
}

impl Document {
    pub fn serialize_with_digest(&self) -> Result<(Vec<u8>, String), ManuscriptError> {
        let ser = self.serialize()?;
        let ser2 = String::from_utf8(ser.clone()).unwrap();
        Ok((ser, sha256::digest(ser2)))
    }

    pub fn serialize(&self) -> Result<Vec<u8>, ManuscriptError> {
        match bincode::serialize(&SerializableDocument::new(self)) {
            Ok(ser) => Ok(ser),
            Err(_) => Err(ManuscriptError::DocumentSerialize),
        }
    }
}

impl<'d> TryFrom<&'d [u8]> for Document {
    type Error = ManuscriptError;

    fn try_from(value: &'d [u8]) -> Result<Self, Self::Error> {
        let maybe_deserialized_document: Result<SerializableDocument, Box<bincode::ErrorKind>> =
            bincode::deserialize(value);
        match maybe_deserialized_document {
            Ok(deserialized) => Ok(Document::from(deserialized)),
            Err(_) => Err(ManuscriptError::DocumentDeserialize),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SerializableDocument {
    manifest: DocumentManifest,
    settings: DocumentSettings,
    chapters: Vec<Chapter>,
    character_sheets: Vec<CharacterSheet>,
}

impl SerializableDocument {
    pub fn new(source: &Document) -> Self {
        let source = source.clone();
        let manifest = source.manifest().clone();
        let settings = source.settings.clone();
        let mut chapters = vec![];
        let mut character_sheets = vec![];

        let chunks = source.chunks();
        for c in chunks {
            let chunk = c.as_any();
            if let Some(downcasted) = chunk.downcast_ref::<Chapter>() {
                chapters.push(downcasted.clone());
            } else if let Some(downcasted) = chunk.downcast_ref::<CharacterSheet>() {
                character_sheets.push(downcasted.clone());
            } else {
                unreachable!()
            }
        }

        Self {
            manifest,
            settings,
            chapters,
            character_sheets,
        }
    }

    pub fn chapters(&self) -> &Vec<Chapter> {
        self.chapters.as_ref()
    }
}

#[allow(clippy::field_reassign_with_default)]
impl From<SerializableDocument> for Document {
    fn from(source: SerializableDocument) -> Self {
        let mut document = Document::default();
        document.manifest = source.manifest.clone();
        document.settings = source.settings.clone();

        for chapter in source.chapters {
            document.add_chunk(chapter);
        }

        for character_sheet in source.character_sheets {
            document.add_chunk(character_sheet);
        }

        document
    }
}

impl From<&Document> for SerializableDocument {
    fn from(source: &Document) -> Self {
        Self::new(source)
    }
}

impl From<Vec<u8>> for Document {
    fn from(source: Vec<u8>) -> Self {
        Document::try_from(source.as_slice()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    fn make_test_document_1() -> Document {
        let mut doc: Document = Document::default();
        doc.add_chunk(Chapter::default());
        doc.add_chunk(Chapter::default());
        doc.add_chunk(Chapter::default());
        doc.add_chunk(CharacterSheet::default());
        assert_eq!(doc.chunks().len(), 4);
        doc
    }

    #[test]
    fn serialize_default() {
        let doc: Document = Document::default();
        let serialized = doc.serialize();
        assert!(serialized.is_ok());
    }

    #[test]
    fn serialize_with_content() {
        let doc = make_test_document_1();
        let serialized = doc.serialize();
        assert!(serialized.is_ok());
    }

    #[test]
    fn deserialize() {
        let mut doc = make_test_document_1();
        let mut chapter_with_content = Chapter::default();
        let chapter_with_content_id = chapter_with_content.id().to_string();
        chapter_with_content.set_title("Test title");
        chapter_with_content.set_buffer(Bytes::from(String::from("Test content")));
        doc.add_chunk(chapter_with_content);

        let all_ids: Vec<&str> = doc.chunks().iter().map(|c| c.id()).collect();
        let serialized = doc.serialize();
        assert!(serialized.is_ok());

        let deserialized = Document::try_from(serialized.unwrap());
        assert!(deserialized.is_ok());
        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized.chunks().len(), 5);

        // Check deserialization preserves ids
        for id in all_ids {
            assert!(deserialized.get_chunk_ref(id).is_some());
        }

        let chap = deserialized.get_chunk_ref(chapter_with_content_id.as_str());
        assert!(chap.is_some());
        assert!(chap.unwrap().title().is_some());
        assert!(chap.unwrap().title().unwrap().eq("Test title"));

        let chap = chap.unwrap().as_any().downcast_ref::<Chapter>();
        let expected_bytes = String::from("Test content").into_bytes();
        let actual_bytes = chap.unwrap().buffer().to_vec();
        assert_eq!(expected_bytes, actual_bytes);
    }
}
