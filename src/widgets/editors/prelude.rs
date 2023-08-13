use crate::{models::DocumentChunk, services::DocumentAction};
use glib::Sender;
use gtk::prelude::WidgetExt;

#[derive(Debug, Clone)]
pub enum EditorWidgetFocusResult {
    Unfocusable,
    Focused,
    NotFocused,
}

pub trait ConstructFromChunk {
    fn new(chunk: &dyn DocumentChunk) -> Self;
}

pub trait EditorWidgetProtocol {
    fn document_action_sender(&self) -> Option<Sender<DocumentAction>> {
        None
    }

    fn editor_widget(&self) -> Option<gtk::Widget> {
        None
    }

    fn side_panel_widget(&self) -> Option<gtk::Widget> {
        None
    }

    fn grab_focus(&self) -> EditorWidgetFocusResult {
        if let Some(widget) = self.editor_widget() {
            match widget.grab_focus() {
                true => EditorWidgetFocusResult::Focused,
                false => EditorWidgetFocusResult::NotFocused,
            }
        } else {
            EditorWidgetFocusResult::Unfocusable
        }
    }
}

impl<T> EditorWidgetProtocol for Box<T>
where
    T: ?Sized + EditorWidgetProtocol,
{
    fn document_action_sender(&self) -> Option<Sender<DocumentAction>> {
        self.as_ref().document_action_sender()
    }

    fn editor_widget(&self) -> Option<gtk::Widget> {
        self.as_ref().editor_widget()
    }

    fn side_panel_widget(&self) -> Option<gtk::Widget> {
        self.as_ref().side_panel_widget()
    }

    fn grab_focus(&self) -> EditorWidgetFocusResult {
        self.as_ref().grab_focus()
    }
}
