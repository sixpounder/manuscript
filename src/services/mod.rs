use crate::{
    config::G_LOG_DOMAIN,
    models::{Chapter, Document, MutableBufferChunk},
};

use bytes::Bytes;
use std::{
    any::Any,
    rc::Rc,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

pub enum DocumentAction {
    UpdateChunkBuffer(String, Bytes),
}

#[derive(Debug)]
pub struct DocumentManager {
    document: Arc<Mutex<Document>>,
    rx: Option<Receiver<DocumentAction>>,
    tx: Rc<Sender<DocumentAction>>,
}

impl DocumentManager {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let tx = Rc::new(tx);
        let document = Arc::new(Mutex::new(Document::default()));

        Self {
            document,
            rx: Some(rx),
            tx,
        }
    }

    pub fn listen(&mut self) {
        let rx = self.rx.take().unwrap();
        let document = Arc::clone(&self.document);
        std::thread::spawn(move || {
            while let Ok(action) = rx.recv() {
                // Process action
                if let Ok(mut lock) = document.lock() {
                    match action {
                        DocumentAction::UpdateChunkBuffer(id, bytes) => {
                            if let Some(chunk) = lock.get_chunk_mut(id.as_str()) {
                                let as_any = Box::new(chunk.as_any_mut());
                                if let Some(mbc) =
                                    as_any.downcast_mut::<Box<dyn MutableBufferChunk>>()
                                {
                                    mbc.set_buffer(bytes);
                                } else {
                                    glib::warn!("An UpdateChunkBuffer was requested on {:?}, but it doesnt implement MutableBufferChunk", as_any);
                                }
                            } else {
                                glib::warn!("An UpdateChunkBuffer was requested on chunk with id {id}, but it was not found");
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
        });
    }

    pub fn spawn_sender(&self) -> Rc<Sender<DocumentAction>> {
        Rc::clone(&self.tx)
    }
}

impl Default for DocumentManager {
    fn default() -> Self {
        DocumentManager::new()
    }
}
