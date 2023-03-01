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
    #[template(resource = "/io/sixpounder/Manuscript/sheet_editor.ui")]
    pub struct ManuscriptCharacterSheetEditor {}

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptCharacterSheetEditor {
        const NAME: &'static str = "ManuscriptCharacterSheetEditor";
        type Type = super::ManuscriptCharacterSheetEditor;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptCharacterSheetEditor {
        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(Vec::new);
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            let imp = obj.imp();
            match pspec.name() {
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let _obj = self.obj();
            match pspec.name() {
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ManuscriptCharacterSheetEditor {}
}

glib::wrapper! {
    pub struct ManuscriptCharacterSheetEditor(ObjectSubclass<imp::ManuscriptCharacterSheetEditor>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ManuscriptCharacterSheetEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl ManuscriptCharacterSheetEditor {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }
}

