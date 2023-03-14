use gtk::{gio, glib, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;
    use glib::ParamSpec;
    use once_cell::sync::Lazy;

    #[derive(Default)]
    pub struct ManuscriptBuffer {}

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptBuffer {
        const NAME: &'static str = "ManuscriptBuffer";
        type Type = super::ManuscriptBuffer;
        type ParentType = gtk::TextBuffer;

        // fn class_init(klass: &mut Self::Class) {
        //     klass.bind_template();
        //     klass.set_layout_manager_type::<gtk::BinLayout>();
        // }

        // fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        //     obj.init_template();
        // }
    }

    impl ObjectImpl for ManuscriptBuffer {
        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(Vec::new);
            PROPERTIES.as_ref()
        }

        // fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
        //     let _obj = self.obj();
        //     match pspec.name() {
        //         _ => unimplemented!(),
        //     }
        // }

        fn set_property(&self, _id: usize, _value: &glib::Value, _pspec: &ParamSpec) {
            // let _obj = self.obj();
            // match pspec.name() {
            //     _ => unimplemented!(),
            // }
        }
    }

    impl TextBufferImpl for ManuscriptBuffer {}
}

glib::wrapper! {
    pub struct ManuscriptBuffer(ObjectSubclass<imp::ManuscriptBuffer>) @extends gtk::TextBuffer;
}

impl Default for ManuscriptBuffer {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ManuscriptBuffer {
    pub fn new(tag_table: Option<gtk::TextTagTable>) -> Self {
        glib::Object::new(&[("tag-table", &tag_table)])
    }
}
