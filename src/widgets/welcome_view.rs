use adw::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/welcome_view.ui")]
    pub struct ManuscriptWelcomeView;

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptWelcomeView {
        const NAME: &'static str = "ManuscriptWelcomeView";
        type Type = super::ManuscriptWelcomeView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptWelcomeView {}
    impl WidgetImpl for ManuscriptWelcomeView {}
}

glib::wrapper! {
    pub struct ManuscriptWelcomeView(ObjectSubclass<imp::ManuscriptWelcomeView>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ManuscriptWelcomeView {
    fn default() -> Self {
        Self::new()
    }
}

impl ManuscriptWelcomeView {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
