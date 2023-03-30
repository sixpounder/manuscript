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
    #[properties(wrapper_type = super::ManuscriptProjectSettings)]
    #[template(resource = "/io/sixpounder/Manuscript/project_settings.ui")]
    pub struct ManuscriptProjectSettings {
        #[property(get, set)]
        pub(super) title: RefCell<String>
    }

    impl Default for ManuscriptProjectSettings {
        fn default() -> Self {
            Self {
                title: RefCell::new(i18n("Project Settings").into())
            }
        }
    }

    impl ManuscriptProjectSettings {}

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptProjectSettings {
        const NAME: &'static str = "ManuscriptProjectSettings";
        type Type = super::ManuscriptProjectSettings;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptProjectSettings {
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

    impl WidgetImpl for ManuscriptProjectSettings {}
}

glib::wrapper! {
    pub struct ManuscriptProjectSettings(ObjectSubclass<imp::ManuscriptProjectSettings>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ManuscriptProjectSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl ManuscriptProjectSettings {
    pub fn new() -> Self {
        glib::Object::builder()
            .build()
    }
}
