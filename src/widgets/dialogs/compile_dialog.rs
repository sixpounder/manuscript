use adw::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;
    // use glib::ParamSpec;
    // use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/dialogs/compile_dialog.ui")]
    pub struct ManuscriptCompileDialog {}

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptCompileDialog {
        const NAME: &'static str = "ManuscriptCompileDialog";
        type Type = super::ManuscriptCompileDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptCompileDialog {
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

    impl WidgetImpl for ManuscriptCompileDialog {}
    impl WindowImpl for ManuscriptCompileDialog {}
    impl AdwWindowImpl for ManuscriptCompileDialog {}
}

glib::wrapper! {
    pub struct ManuscriptCompileDialog(ObjectSubclass<imp::ManuscriptCompileDialog>)
        @extends adw::Window, gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptCompileDialog {
    pub fn new(parent: &gtk::Window) -> Self {
        glib::Object::builder()
            .property("modal", true)
            .property("transient-for", parent)
            .build()
    }
}
