use crate::{config::G_LOG_DOMAIN, models::*, services::DocumentAction};
use adw::subclass::prelude::*;
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
    #[template(resource = "/io/sixpounder/Manuscript/project_layout.ui")]
    pub struct ManuscriptProjectLayout {
        pub(super) sender: RefCell<Option<Sender<DocumentAction>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptProjectLayout {
        const NAME: &'static str = "ManuscriptProjectLayout";
        type Type = super::ManuscriptProjectLayout;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptProjectLayout {
        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| vec![]);
            PROPERTIES.as_ref()
        }

        // fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
        //     let obj = self.obj();
        //     let imp = obj.imp();
        //     match pspec.name() {
        //         "locked" => imp.locked.get().to_value(),
        //         "chunk-id" => imp.chunk_id.borrow().to_value(),
        //         "text-buffer" => imp.text_buffer.borrow().to_value(),
        //         _ => unimplemented!(),
        //     }
        // }

        // fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
        //     let _obj = self.obj();
        //     match pspec.name() {
        //         "locked" => self.locked.set(value.get::<bool>().unwrap()),
        //         "chunk-id" => {
        //             self.chunk_id
        //                 .replace(value.get::<Option<String>>().unwrap());
        //         }
        //         _ => unimplemented!(),
        //     }
        // }
    }

    impl WidgetImpl for ManuscriptProjectLayout {}
}

glib::wrapper! {
    pub struct ManuscriptProjectLayout(ObjectSubclass<imp::ManuscriptProjectLayout>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptProjectLayout {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn load_document(&self, document: &Document) {
        glib::debug!("Setting document on project layout widget");
    }

    pub fn add_chunk<C: DocumentChunk + ?Sized>(&self, chunk: &C) {
        glib::debug!("Adding chunk with id {} from project layout", chunk.id());
    }

    pub fn remove_chunk<S: ToString>(&self, chunk_id: S) {
        glib::debug!(
            "Removing chunk with id {} from project layout",
            chunk_id.to_string()
        );
    }
}
