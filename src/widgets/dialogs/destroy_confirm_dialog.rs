use adw::subclass::prelude::*;
use gtk::{gio, glib};
// use glib_macros::Properties;

mod imp {
    use super::*;
    // use glib::ParamSpec;
    // use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/dialogs/destroy_confirm_dialog.ui")]
    pub struct ManuscriptDestroyConfirmDialog {}

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptDestroyConfirmDialog {
        const NAME: &'static str = "ManuscriptDestroyConfirmDialog";
        type Type = super::ManuscriptDestroyConfirmDialog;
        type ParentType = adw::MessageDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptDestroyConfirmDialog {}

    impl WidgetImpl for ManuscriptDestroyConfirmDialog {}
    impl WindowImpl for ManuscriptDestroyConfirmDialog {}
    impl MessageDialogImpl for ManuscriptDestroyConfirmDialog {}
}

glib::wrapper! {
    pub struct ManuscriptDestroyConfirmDialog(ObjectSubclass<imp::ManuscriptDestroyConfirmDialog>)
        @extends adw::MessageDialog, gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptDestroyConfirmDialog {
    pub fn new(parent: &gtk::Window) -> Self {
        glib::Object::builder()
            .property("modal", true)
            .property("transient-for", parent)
            .build()
    }
}
