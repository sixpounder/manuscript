use super::chunk::{Chapter, CharacterSheet, DocumentManifest};
use super::prelude::{ChunkType, DocumentChunk, ManuscriptError};
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::HashMap, ops::Deref};

#[derive(Clone, Default)]
pub struct Document {
    manifest: DocumentManifest,
    chunks: HashMap<String, Box<dyn DocumentChunk>>,
}

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

impl From<SerializableDocument> for Document {
    fn from(_: SerializableDocument) -> Self {
        // TODO: implement the conversion
        unimplemented!()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialize() {
        let doc: Document = Document::default();
        let serialized = doc.serialize();
        assert!(serialized.is_ok());
    }
}
