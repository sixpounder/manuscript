use adw::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/command_palette.ui")]
    pub struct ManuscriptCommandPalette {}

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptCommandPalette {
        const NAME: &'static str = "ManuscriptCommandPalette";
        type Type = super::ManuscriptCommandPalette;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptCommandPalette {}

    impl WidgetImpl for ManuscriptCommandPalette {}
}

glib::wrapper! {
    pub struct ManuscriptCommandPalette(ObjectSubclass<imp::ManuscriptCommandPalette>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ManuscriptCommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl ManuscriptCommandPalette {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
