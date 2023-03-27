use super::compile::CompilePanel;
use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

mod imp {
    use super::*;
    // use glib::ParamSpec;
    // use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/dialogs/pdf_panel.ui")]
    pub struct ManuscriptCompilePdfPanel {}

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptCompilePdfPanel {
        const NAME: &'static str = "ManuscriptCompilePdfPanel";
        type Type = super::ManuscriptCompilePdfPanel;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptCompilePdfPanel {
        // fn properties() -> &'static [gtk::glib::ParamSpec] {
        //     static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(Vec::new);
        //     PROPERTIES.as_ref()
        // }

        // fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
        //     let _obj = self.obj();
        //     match pspec.name() {
        //         _ => unimplemented!(),
        //     }
        // }

        // fn set_property(&self, _id: usize, _value: &glib::Value, pspec: &ParamSpec) {
        //     let _obj = self.obj();
        //     match pspec.name() {
        //         _ => unimplemented!(),
        //     }
        // }
    }

    impl WidgetImpl for ManuscriptCompilePdfPanel {}
}

glib::wrapper! {
    pub struct ManuscriptCompilePdfPanel(ObjectSubclass<imp::ManuscriptCompilePdfPanel>)
        @extends adw::Window, gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ManuscriptCompilePdfPanel {
    fn default() -> Self {
        Self::new()
    }
}


impl ManuscriptCompilePdfPanel {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }
}

impl CompilePanel for ManuscriptCompilePdfPanel {}
