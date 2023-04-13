use crate::services::DocumentAction;
use glib::Sender;

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
}
