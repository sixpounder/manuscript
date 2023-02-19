use crate::{config::G_LOG_DOMAIN, models::*, services::DocumentAction};
use adw::subclass::prelude::*;
use bytes::Bytes;
use gtk::{
    gio, glib,
    glib::{clone, Sender},
    prelude::*,
};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/text_editor.ui")]
    pub struct ManuscriptTextEditor {
        pub(super) sender: RefCell<Option<Sender<DocumentAction>>>,
        pub(super) chunk_id: RefCell<Option<String>>,
        pub(super) text_buffer: RefCell<Option<gtk::TextBuffer>>,
        pub(super) locked: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptTextEditor {
        const NAME: &'static str = "ManuscriptTextEditor";
        type Type = super::ManuscriptTextEditor;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptTextEditor {
        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new("locked", "", "", false, ParamFlags::READWRITE),
                    ParamSpecString::new("chunk-id", "", "", None, ParamFlags::READWRITE),
                    ParamSpecObject::new(
                        "text-buffer",
                        "",
                        "",
                        Option::<gtk::TextBuffer>::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            let imp = obj.imp();
            match pspec.name() {
                "locked" => imp.locked.get().to_value(),
                "chunk-id" => imp.chunk_id.borrow().to_value(),
                "text-buffer" => imp.text_buffer.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let _obj = self.obj();
            match pspec.name() {
                "locked" => self.locked.set(value.get::<bool>().unwrap()),
                "chunk-id" => {
                    self.chunk_id
                        .replace(value.get::<Option<String>>().unwrap());
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ManuscriptTextEditor {}
}

glib::wrapper! {
    pub struct ManuscriptTextEditor(ObjectSubclass<imp::ManuscriptTextEditor>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptTextEditor {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn init(&self, chunk_id: String, buffer: Option<Bytes>) {
        let imp = self.imp();

        imp.chunk_id.replace(Some(chunk_id));
        self.set_buffer(buffer);
    }

    fn set_buffer(&self, value: Option<Bytes>) {
        let imp = self.imp();
        let text_buffer = gtk::TextBuffer::new(None);
        text_buffer.set_text(
            String::from_utf8(value.unwrap_or(Bytes::new()).slice(..).to_vec())
                .unwrap()
                .as_str(),
        );
        imp.text_buffer.replace(Some(text_buffer));

        self.connect_text_buffer();
    }

    pub fn text_buffer(&self) -> std::cell::Ref<Option<gtk::TextBuffer>> {
        self.imp().text_buffer.borrow()
    }

    fn connect_text_buffer(&self) {
        match self.text_buffer().as_ref() {
            Some(buffer) => {
                buffer.connect_changed(clone!(@strong self as this => move |buf| {
                    this.on_buffer_changed(buf).expect("Could not update chunk")
                }));
            }
            None => (),
        }
    }

    fn on_buffer_changed(&self, buf: &gtk::TextBuffer) -> Result<(), ManuscriptError> {
        let chunk_id = self.imp().chunk_id.borrow();
        if let Some(chunk_id) = chunk_id.as_ref() {
            let start_iter = buf.start_iter();
            let end_iter = buf.end_iter();
            let new_bytes = Bytes::from(buf.text(&start_iter, &end_iter, true).to_string());
            let tx = self.imp().sender.borrow();
            let tx = tx.as_ref().unwrap();
            if let Ok(_) = tx.send(DocumentAction::UpdateChunkBuffer(
                chunk_id.to_string(),
                new_bytes,
            )) {
                Ok(())
            } else {
                Err(ManuscriptError::ChunkUnavailable)
            }
        } else {
            glib::warn!("No chunk id is set on this ManuscriptTextEditor");
            Err(ManuscriptError::ChunkUnavailable)
        }
    }
}
