use crate::{
    config::G_LOG_DOMAIN,
    models::{
        Chapter, CharacterSheet, Document, DocumentChunk, ManuscriptError, ManuscriptResult,
        MutableBufferChunk,
    },
};
use adw::subclass::prelude::*;
use bytes::Bytes;
use glib::{clone, MainContext, ObjectExt, Receiver, Sender};
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

pub enum DocumentAction {
    AddChapter(Chapter),
    AddCharacterSheet(CharacterSheet),
    UpdateChunkBuffer(String, Bytes),
}

mod imp {
    use super::*;
    use glib::{
        subclass::{object::ObjectImpl, signal::Signal},
        types::StaticType,
    };
    use once_cell::sync::Lazy;

    pub struct ManuscriptDocumentManager {
        pub(super) document: Arc<Mutex<Document>>,
        pub(super) rx: RefCell<Option<Receiver<DocumentAction>>>,
        pub(super) tx: Sender<DocumentAction>,
    }

    impl Default for ManuscriptDocumentManager {
        fn default() -> Self {
            let (tx, rx) = MainContext::channel(glib::PRIORITY_DEFAULT);
            let document = Arc::new(Mutex::new(Document::default()));

            Self {
                document,
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
                    Signal::builder("chunk-added")
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder("chunk-removed")
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder("chunk-updated")
                        .param_types([String::static_type()])
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
    pub fn new() -> Self {
        let obj: Self = glib::Object::new::<Self>(&[]);
        obj.listen();
        obj
    }

    pub fn with_document<F, T>(&self, f: F) -> Result<T, ManuscriptError>
    where
        F: Fn(&Document) -> Result<T, ManuscriptError>,
    {
        if let Ok(lock) = self.imp().document.lock() {
            f(&lock)
        } else {
            Err(ManuscriptError::DocumentLock)
        }
    }

    pub fn with_document_mut<F, T>(&self, f: F) -> Result<T, ManuscriptError>
    where
        F: FnOnce(&mut Document) -> Result<T, ManuscriptError>,
    {
        if let Ok(mut lock) = self.imp().document.lock() {
            f(&mut lock)
        } else {
            Err(ManuscriptError::DocumentLock)
        }
    }

    pub fn set_document(&self, document: Document) -> ManuscriptResult<()> {
        if let Ok(mut lock) = self.imp().document.lock() {
            *lock = document;
            self.emit_by_name::<()>("document-loaded", &[]);
            Ok(())
        } else {
            Err(ManuscriptError::DocumentLock)
        }
    }

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
        if let Ok(mut lock) = self.imp().document.lock() {
            match action {
                DocumentAction::AddChapter(value) => {
                    let id = value.id().to_string();
                    lock.add_chunk(value);
                    self.emit_by_name::<()>("chunk-added", &[&id]);
                }
                DocumentAction::AddCharacterSheet(value) => {
                    let id = value.id().to_string();
                    lock.add_chunk(value);
                    self.emit_by_name::<()>("chunk-added", &[&id]);
                }
                DocumentAction::UpdateChunkBuffer(id, bytes) => {
                    if let Some(chunk) = lock.get_chunk_mut(id.as_str()) {
                        let id = chunk.id().to_string();
                        let as_any = Box::new(chunk.as_any_mut());
                        if let Some(mbc) = as_any.downcast_mut::<Box<dyn MutableBufferChunk>>() {
                            mbc.set_buffer(bytes);
                            self.emit_by_name::<()>("chunk-updated", &[&id]);
                        } else {
                            glib::warn!("An UpdateChunkBuffer was requested on {:?}, but it doesnt implement MutableBufferChunk", as_any);
                        }
                    } else {
                        glib::warn!("An UpdateChunkBuffer was requested on chunk with id {id}, but it was not found");
                    }
                }
            }
        }
    }

    pub fn add_chunk<C: DocumentChunk + 'static>(&self, value: C) {
        let id = value.id().to_string();
        if let Ok(_) = self.with_document_mut(move |document| {
            document.add_chunk(value);
            Ok(())
        }) {
            self.emit_by_name::<()>("chunk-added", &[&id]);
        }
    }

    pub fn remove_chunk(&self, id: &String) -> Option<Box<dyn DocumentChunk>> {
        let imp = self.imp();
        if let Ok(mut lock) = imp.document.lock() {
            let removed = lock.remove_chunk(id);
            drop(lock);
            self.emit_by_name::<()>("chunk-removed", &[id]);
            removed
        } else {
            None
        }
    }

    pub fn document_ref(&self) -> &std::sync::Mutex<Document> {
        self.imp().document.as_ref()
    }

    pub fn action_sender(&self) -> Sender<DocumentAction> {
        self.imp().tx.clone()
    }
}
