use crate::models::{
    Chapter, CharacterSheet, Document, DocumentChunk, DocumentSettings, ManuscriptError,
    ManuscriptResult, MutableBufferChunk,
};
use adw::subclass::prelude::*;
use bytes::{Buf, Bytes, BytesMut};
use glib::{clone, MainContext, ObjectExt, Receiver, Sender};
use std::{
    cell::RefCell,
    fs::File,
    io::{prelude::*, BufReader, Read},
    sync::{LockResult, RwLock},
};

const G_LOG_DOMAIN: &str = "ManuscriptDocumentManager";

#[derive(Debug, Clone)]
pub struct BufferStats {
    words_count: u64,
    reading_time: (u64, u64),
}

impl BufferStats {
    pub fn new(words_count: u64, reading_time: (u64, u64)) -> Self {
        Self {
            words_count,
            reading_time,
        }
    }
}

impl std::fmt::Display for BufferStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}:{}",
            self.words_count, self.reading_time.0, self.reading_time.1
        )
    }
}

type ChunkUpdateFunc = dyn FnOnce(&mut dyn DocumentChunk);

pub enum DocumentAction {
    SetTitle(String),
    AddChapter(Chapter),
    AddCharacterSheet(CharacterSheet),
    SelectChunk(String),
    UpdateChunkBuffer(String, Bytes),
    UpdateChunkBufferStats(String, BufferStats),
    UpdateChunk(String),
    UpdateChunkWith(String, Box<ChunkUpdateFunc>),
}

impl std::fmt::Display for DocumentAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetTitle(title) => write!(f, "DocumentAction::SetTitle({} bytes)", title.len()),
            Self::AddChapter(item) => write!(f, "DocumentAction::AddChapter(#{})", item.id()),
            Self::AddCharacterSheet(item) => {
                write!(f, "DocumentAction::AddCharacterSheet(#{})", item.id())
            }
            Self::SelectChunk(id) => write!(f, "DocumentAction::SelectChunk(#{})", id),
            Self::UpdateChunkBuffer(id, bytes) => write!(
                f,
                "DocumentAction::UpdateChunkBuffer(#{} - {} bytes)",
                id,
                bytes.len()
            ),
            Self::UpdateChunkBufferStats(id, stats) => write!(
                f,
                "DocumentAction::UpdateChunkBufferStats(#{} - {})",
                id, stats
            ),
            Self::UpdateChunk(id) => write!(f, "DocumentAction::UpdateChunk(#{id})"),
            Self::UpdateChunkWith(id, _func) => {
                write!(f, "DocumentAction::UpdateChunkWith(#{id}, function)")
            }
        }
    }
}

mod imp {
    use super::*;
    use glib::{
        subclass::{object::ObjectImpl, signal::Signal},
        types::StaticType,
    };
    use once_cell::sync::Lazy;

    pub struct ManuscriptDocumentManager {
        pub(super) document: RwLock<Option<Document>>,
        pub(super) backend_path: RefCell<Option<String>>,
        pub(super) rx: RefCell<Option<Receiver<DocumentAction>>>,
        pub(super) tx: Sender<DocumentAction>,
    }

    impl Default for ManuscriptDocumentManager {
        fn default() -> Self {
            let (tx, rx) = MainContext::channel(glib::PRIORITY_DEFAULT);

            Self {
                document: RwLock::new(None),
                backend_path: RefCell::new(None),
                rx: RefCell::new(Some(rx)),
                tx,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptDocumentManager {
        const NAME: &'static str = "ManuscriptDocumentManager";
        type Type = super::DocumentManager;
        type ParentType = glib::Object;

        fn new() -> Self {
            Self::default()
        }
    }

    impl ObjectImpl for ManuscriptDocumentManager {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("document-loaded").build(),
                    Signal::builder("document-unloaded").build(),
                    Signal::builder("title-set").build(),
                    Signal::builder("chunk-added")
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder("chunk-removed")
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder("chunk-selected")
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder("chunk-updated")
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder("chunk-stats-updated")
                        .param_types([
                            String::static_type(),
                            u64::static_type(),
                            u64::static_type(),
                            u64::static_type(),
                        ])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }
}

glib::wrapper! {
    pub struct DocumentManager(ObjectSubclass<imp::ManuscriptDocumentManager>);
}

impl Default for DocumentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentManager {
    fn listen(&self) {
        let rx = self.imp().rx.borrow_mut().take().unwrap();
        rx.attach(
            None,
            clone!(@strong self as this => move |action| {
                this.process_action(action);
                glib::Continue(true)
            }),
        );
    }

    fn process_action(&self, action: DocumentAction) {
        glib::trace!("{}::{}", G_LOG_DOMAIN, action);
        match action {
            DocumentAction::SetTitle(new_title) => {
                if let Ok(mut lock) = self.imp().document.write() {
                    if let Some(document) = lock.as_mut() {
                        document.set_title(Some(new_title));
                    }
                }
                self.emit_by_name::<()>("title-set", &[]);
            }
            DocumentAction::AddChapter(value) => {
                let id = value.id().to_string();
                if let Ok(mut lock) = self.imp().document.write() {
                    if let Some(document) = lock.as_mut() {
                        document.add_chunk(value);
                    }
                }
                self.emit_by_name::<()>("chunk-added", &[&id]);
            }
            DocumentAction::AddCharacterSheet(value) => {
                let id = value.id().to_string();
                if let Ok(mut lock) = self.imp().document.write() {
                    if let Some(document) = lock.as_mut() {
                        document.add_chunk(value);
                    }
                }
                self.emit_by_name::<()>("chunk-added", &[&id]);
            }
            DocumentAction::SelectChunk(id) => {
                if let Ok(lock) = self.imp().document.read() {
                    if let Some(document) = lock.as_ref() {
                        if let Some(_chunk) = document.get_chunk_ref(id.as_str()) {
                            self.emit_by_name::<()>("chunk-selected", &[&id]);
                        } else {
                            glib::g_warning!(G_LOG_DOMAIN, "DocumentManager -> Tried to select chunk {} but it was not found in document", id);
                        }
                    }
                }
            }
            DocumentAction::UpdateChunkBuffer(id, bytes) => {
                if let Ok(mut lock) = self.imp().document.write() {
                    if let Some(document) = lock.as_mut() {
                        if let Some(chunk) = document.get_chunk_mut(id.as_str()) {
                            let as_any = chunk.as_any_mut();
                            if let Some(mbc) = as_any.downcast_mut::<Chapter>() {
                                mbc.set_buffer(bytes);
                            } else {
                                glib::g_warning!(G_LOG_DOMAIN, "An UpdateChunkBuffer was requested on {:?}, but it doesnt implement MutableBufferChunk", as_any);
                            }
                        } else {
                            glib::g_warning!(G_LOG_DOMAIN, "An UpdateChunkBuffer was requested on chunk with id {id}, but it was not found");
                        }
                    }
                }

                self.emit_by_name::<()>("chunk-updated", &[&id]);
            }
            DocumentAction::UpdateChunkBufferStats(id, stats) => {
                self.emit_by_name::<()>(
                    "chunk-stats-updated",
                    &[
                        &id,
                        &stats.words_count,
                        &stats.reading_time.0,
                        &stats.reading_time.1,
                    ],
                );
            }
            DocumentAction::UpdateChunk(_id) => (),
            DocumentAction::UpdateChunkWith(id, func) => {
                if let Ok(mut lock) = self.imp().document.write() {
                    if let Some(document) = lock.as_mut() {
                        if let Some(chunk) = document.get_chunk_mut(id.as_str()) {
                            func(chunk);
                            drop(lock);
                            self.emit_by_name::<()>("chunk-updated", &[&id]);
                        }
                    }
                }
            }
        }
    }

    pub fn new() -> Self {
        let obj: Self = glib::Object::new::<Self>(&[]);
        obj.listen();
        obj
    }

    pub fn has_document(&self) -> bool {
        if let Ok(lock) = self.imp().document.read() {
            lock.is_some()
        } else {
            false
        }
    }

    pub fn document_guard(&self) -> &RwLock<Option<Document>> {
        &self.imp().document
    }

    pub fn document_settings(&self) -> Result<DocumentSettings, ManuscriptError> {
        if let Ok(lock) = self.document_guard().read() {
            if let Some(document) = lock.as_ref() {
                Ok(document.settings().clone())
            } else {
                Err(ManuscriptError::NoDocument)
            }
        } else {
            Err(ManuscriptError::DocumentLock)
        }
    }

    pub fn with_document<F, T>(&self, f: F) -> Result<T, ManuscriptError>
    where
        F: Fn(&Document) -> Result<T, ManuscriptError>,
    {
        if let Ok(lock) = self.document_guard().read() {
            if let Some(document) = lock.as_ref() {
                f(document)
            } else {
                Err(ManuscriptError::NoDocument)
            }
        } else {
            Err(ManuscriptError::DocumentLock)
        }
    }

    pub fn with_document_mut<F, T>(&self, f: F) -> Result<T, ManuscriptError>
    where
        F: FnOnce(&mut Document) -> Result<T, ManuscriptError>,
    {
        if let Ok(mut lock) = self.document_guard().write() {
            if let Some(document) = lock.as_mut() {
                f(document)
            } else {
                Err(ManuscriptError::DocumentLock)
            }
        } else {
            Err(ManuscriptError::NoDocument)
        }
    }

    pub fn set_document(&self, document: Document) -> ManuscriptResult<()> {
        if let Ok(mut lock) = self.document_guard().write() {
            *lock = Some(document);
            drop(lock);
            self.emit_by_name::<()>("document-loaded", &[]);
            Ok(())
        } else {
            Err(ManuscriptError::DocumentLock)
        }
    }

    pub fn unset_document(&self) -> ManuscriptResult<()> {
        if let Ok(mut lock) = self.document_guard().write() {
            if lock.is_some() {
                *lock = None;
                drop(lock);
                *self.backend_path_mut() = None;
                self.emit_by_name::<()>("document-unloaded", &[]);
            }
            Ok(())
        } else {
            Err(ManuscriptError::DocumentLock)
        }
    }

    pub fn add_chunk<C: DocumentChunk + 'static>(&self, value: C) {
        let id = value.id().to_string();
        let add_chunk_result = self.with_document_mut(move |document| {
            document.add_chunk(value);
            Ok(())
        });
        if add_chunk_result.is_ok() {
            self.emit_by_name::<()>("chunk-added", &[&id]);
        }
    }

    pub fn remove_chunk(&self, id: &String) -> Option<Box<dyn DocumentChunk>> {
        let imp = self.imp();
        if let Ok(mut lock) = imp.document.write() {
            if let Some(document) = lock.as_mut() {
                let removed = document.remove_chunk(id);
                drop(lock);
                self.emit_by_name::<()>("chunk-removed", &[id]);
                removed
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn document_ref(&self) -> LockResult<std::sync::RwLockReadGuard<Option<Document>>> {
        self.imp().document.read()
    }

    pub fn document_mut(&self) -> LockResult<std::sync::RwLockWriteGuard<Option<Document>>> {
        self.imp().document.write()
    }

    pub fn action_sender(&self) -> Sender<DocumentAction> {
        self.imp().tx.clone()
    }

    pub fn load_document(&self, path: String) -> ManuscriptResult<()> {
        if self.unset_document().is_ok() {
            let file = File::open(path.as_str()).expect("Unable to open file");
            let mut buf: BytesMut = BytesMut::with_capacity(4096);
            let mut reader = BufReader::new(file);
            while let Ok(bytes_read) = reader.read(buf.as_mut()) {
                if bytes_read == 0 {
                    break;
                } else {
                    buf.advance(bytes_read);
                }
            }
            if let Ok(document) = Document::try_from(buf.to_vec()) {
                self.set_document(document).expect("Could not set document");
                Ok(())
            } else {
                Err(ManuscriptError::DocumentDeserialize)
            }
        } else {
            Err(ManuscriptError::Reason("Could not unload document"))
        }
    }

    pub fn sync(&self) -> ManuscriptResult<usize> {
        if let Some(backend_file) = self.backend_path().as_ref() {
            self.with_document_mut(move |document| {
                if let Ok(serialized) = document.serialize() {
                    let mut f = File::create(backend_file.as_str()).expect("Unable to create file");
                    if f.write_all(serialized.as_slice()).is_ok() {
                        Ok(serialized.len())
                    } else {
                        Err(ManuscriptError::Save)
                    }
                } else {
                    Err(ManuscriptError::DocumentSerialize)
                }
            })
        } else {
            Err(ManuscriptError::NoBackend)
        }
    }

    pub fn backend_path(&self) -> std::cell::Ref<Option<String>> {
        self.imp().backend_path.borrow()
    }

    pub fn backend_path_mut(&self) -> std::cell::RefMut<Option<String>> {
        self.imp().backend_path.borrow_mut()
    }

    pub fn set_backend_path(&self, path: String) {
        self.imp().backend_path.replace(Some(path));
    }

    pub fn has_backend(&self) -> bool {
        self.backend_path().is_some()
    }
}
