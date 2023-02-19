use crate::models::{BufferChunk, MutableBufferChunk};
use bytes::Bytes;
use gtk::prelude::TextBufferExt;

impl From<&dyn BufferChunk> for gtk::TextBuffer {
    fn from(source: &dyn BufferChunk) -> Self {
        bytes_to_text_buffer(source.buffer().slice(..))
    }
}

impl From<Box<dyn BufferChunk>> for gtk::TextBuffer {
    fn from(source: Box<dyn BufferChunk>) -> Self {
        bytes_to_text_buffer(source.buffer().slice(..))
    }
}

impl From<&dyn MutableBufferChunk> for gtk::TextBuffer {
    fn from(source: &dyn MutableBufferChunk) -> Self {
        bytes_to_text_buffer(source.buffer().slice(..))
    }
}

impl From<Box<dyn MutableBufferChunk>> for gtk::TextBuffer {
    fn from(source: Box<dyn MutableBufferChunk>) -> Self {
        bytes_to_text_buffer(source.buffer().slice(..))
    }
}

impl From<Box<&dyn MutableBufferChunk>> for gtk::TextBuffer {
    fn from(source: Box<&dyn MutableBufferChunk>) -> Self {
        bytes_to_text_buffer(source.buffer().slice(..))
    }
}

pub fn bytes_to_text_buffer(source: Bytes) -> gtk::TextBuffer {
    let text_buffer = gtk::TextBuffer::new(None);
    text_buffer.set_text(String::from_utf8(source.to_vec()).unwrap().as_str());
    text_buffer
}
