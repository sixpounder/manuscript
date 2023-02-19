use super::chunk::{Chapter, CharacterSheet, DocumentManifest};
use super::prelude::{ChunkType, DocumentChunk, ManuscriptError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Document {
    manifest: DocumentManifest,
    chunks: HashMap<String, Box<dyn DocumentChunk>>,
}

impl glib::StaticType for Document {
    fn static_type() -> glib::Type {
        glib::Type::OBJECT
    }
}

unsafe impl Send for Document {}

impl Document {
    pub fn add_chunk<C: DocumentChunk + 'static>(&mut self, value: C) {
        self.chunks
            .insert(String::from(value.id()), Box::new(value));
    }

    pub fn remove_chunk(&mut self, id: &String) -> Option<Box<dyn DocumentChunk>> {
        self.chunks.remove(id)
    }

    pub fn get_chunk_ref(&self, id: &str) -> Option<&Box<dyn DocumentChunk>> {
        self.chunks.get(id)
    }

    pub fn get_chunk_mut(&mut self, id: &str) -> Option<&mut Box<dyn DocumentChunk>> {
        self.chunks.get_mut(id)
    }

    pub fn manifest(&self) -> &DocumentManifest {
        &self.manifest
    }

    pub fn chunks(&self) -> Vec<&Box<dyn DocumentChunk>> {
        self.chunks
            .values()
            .collect::<Vec<&Box<dyn DocumentChunk>>>()
    }

    pub fn chunks_by_type_ref(&self, ty: ChunkType) -> Vec<&Box<dyn DocumentChunk>> {
        self.chunks
            .values()
            .filter(|v| v.chunk_type() == ty)
            .collect::<Vec<&Box<dyn DocumentChunk>>>()
    }

    pub fn chunks_by_type(&mut self, ty: ChunkType) -> Vec<&Box<dyn DocumentChunk>> {
        self.chunks
            .values()
            .filter(|v| v.chunk_type() == ty)
            .collect::<Vec<&Box<dyn DocumentChunk>>>()
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
    chapters: Vec<Chapter>,
    character_sheets: Vec<CharacterSheet>,
}

impl SerializableDocument {
    pub fn new(source: &Document) -> Self {
        let source = source.clone();
        let manifest = source.manifest().clone();
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
            chapters,
            character_sheets,
        }
    }

    pub fn chapters(&self) -> &Vec<Chapter> {
        self.chapters.as_ref()
    }
}

impl From<SerializableDocument> for Document {
    fn from(source: SerializableDocument) -> Self {
        let mut document = Document::default();
        document.manifest = source.manifest.clone();
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
        let doc = make_test_document_1();
        let all_ids: Vec<&str> = doc.chunks().iter().map(|c| c.id()).collect();
        let serialized = doc.serialize();
        assert!(serialized.is_ok());

        let deserialized = Document::try_from(serialized.unwrap());
        assert!(deserialized.is_ok());
        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized.chunks().len(), 4);

        // Check deserialization preserves ids
        for id in all_ids {
            assert!(deserialized.get_chunk_ref(id).is_some());
        }
    }
}
