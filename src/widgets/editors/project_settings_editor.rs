use crate::services::i18n::i18n;
use crate::{
    models::{DocumentChunk, DocumentManifest},
    services::DocumentAction,
};
use adw::subclass::prelude::*;
use glib_macros::Properties;
use gtk::{gio, glib::Sender, prelude::*};
use std::cell::RefCell;

#[allow(unused)]
const G_LOG_DOMAIN: &str = "ManuscriptProjectSettings";

mod imp {
    use super::*;
    use glib::ParamSpec;

    #[derive(Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::ManuscriptProjectSettingsEditor)]
    #[template(resource = "/io/sixpounder/Manuscript/editors/project_settings_editor.ui")]
    pub struct ManuscriptProjectSettingsEditor {
        pub(super) sender: RefCell<Option<Sender<DocumentAction>>>,

        #[property(get, set)]
        pub(super) heading: RefCell<String>,

        #[property(get, set)]
        pub(super) title: RefCell<String>,

        #[property(get, set)]
        pub(super) author: RefCell<String>,
    }

    impl Default for ManuscriptProjectSettingsEditor {
        fn default() -> Self {
            Self {
                sender: RefCell::default(),
                heading: RefCell::new(i18n("Project Settings")),
                title: RefCell::default(),
                author: RefCell::default(),
            }
        }
    }

    impl ManuscriptProjectSettingsEditor {}

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptProjectSettingsEditor {
        const NAME: &'static str = "ManuscriptProjectSettingsEditor";
        type Type = super::ManuscriptProjectSettingsEditor;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptProjectSettingsEditor {
        fn properties() -> &'static [gtk::glib::ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
    }

    impl WidgetImpl for ManuscriptProjectSettingsEditor {}
}

glib::wrapper! {
    pub struct ManuscriptProjectSettingsEditor(ObjectSubclass<imp::ManuscriptProjectSettingsEditor>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptProjectSettingsEditor {
    pub fn new(manifest: &DocumentManifest, sender: Option<Sender<DocumentAction>>) -> Self {
        let obj: Self = glib::Object::builder().build();
        *obj.imp().sender.borrow_mut() = sender;
        obj.set_title(i18n("Project settings"));
        obj.set_author(manifest.author());

        obj
    }
}
