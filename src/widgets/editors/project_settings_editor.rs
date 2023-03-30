use crate::services::i18n::i18n;
use adw::subclass::prelude::*;
use glib_macros::Properties;
use gtk::{gio, glib, prelude::*};
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
        #[property(get, set)]
        pub(super) title: RefCell<String>,
    }

    impl Default for ManuscriptProjectSettingsEditor {
        fn default() -> Self {
            Self {
                title: RefCell::new(i18n("Project Settings")),
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

impl Default for ManuscriptProjectSettingsEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl ManuscriptProjectSettingsEditor {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
