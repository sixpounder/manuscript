use crate::models::{BufferChunk, MutableBufferChunk};
use bytes::Bytes;
use gtk::prelude::TextBufferExt;

impl From<&dyn BufferChunk> for gtk::TextBuffer {
    fn from(source: &dyn BufferChunk) -> Self {
        text_buffer_from_bytes(source.buffer().slice(..))
    }
}

impl From<Box<dyn BufferChunk>> for gtk::TextBuffer {
    fn from(source: Box<dyn BufferChunk>) -> Self {
        text_buffer_from_bytes(source.buffer().slice(..))
    }
}

impl From<&dyn MutableBufferChunk> for gtk::TextBuffer {
    fn from(source: &dyn MutableBufferChunk) -> Self {
        text_buffer_from_bytes(source.buffer().slice(..))
    }
}

impl From<Box<dyn MutableBufferChunk>> for gtk::TextBuffer {
    fn from(source: Box<dyn MutableBufferChunk>) -> Self {
        text_buffer_from_bytes(source.buffer().slice(..))
    }
}

impl From<Box<&dyn MutableBufferChunk>> for gtk::TextBuffer {
    fn from(source: Box<&dyn MutableBufferChunk>) -> Self {
        text_buffer_from_bytes(source.buffer().slice(..))
    }
}

/// Transforms the source `Bytes` into a `gtk::TextBuffer`. This is currently done
/// by taking the raw text inside the read bytes, but could be done in a more sofisticated fashion
/// like parsing tags etc...
pub fn text_buffer_from_bytes(source: Bytes) -> gtk::TextBuffer {
    let text_buffer = gtk::TextBuffer::new(None);
    text_buffer.set_text(String::from_utf8(source.to_vec()).unwrap().as_str());
    text_buffer
}

/// Transforms the source `gtk::TextBuffer` into `Bytes`. This is currently done
/// by taking the raw text inside the buffer, but could be done in a more sofisticated fashion
/// like parsing tags etc...
pub fn bytes_from_text_buffer(source: &gtk::TextBuffer) -> Bytes {
    let start_iter = source.start_iter();
    let end_iter = source.end_iter();
    Bytes::from(source.text(&start_iter, &end_iter, true).to_string())
}
