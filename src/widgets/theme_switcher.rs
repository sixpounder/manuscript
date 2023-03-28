use crate::services::ManuscriptSettings;
use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

pub struct Theme;

impl Theme {
    pub fn current() -> String {
        ManuscriptSettings::default().color_scheme()
    }

    pub fn set_current(theme: String) {
        ManuscriptSettings::default().set_color_scheme(theme);
    }
}

mod imp {
    use super::*;
    // use once_cell::sync::Lazy;

    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/theme_switcher.ui")]
    pub struct ManuscriptThemeSwitcher {
        #[template_child]
        pub(super) system_selector: TemplateChild<gtk::CheckButton>,

        #[template_child]
        pub(super) light_selector: TemplateChild<gtk::CheckButton>,

        #[template_child]
        pub(super) sepia_selector: TemplateChild<gtk::CheckButton>,

        #[template_child]
        pub(super) dark_selector: TemplateChild<gtk::CheckButton>,

        pub(super) settings: ManuscriptSettings,

        pub(super) style_manager: adw::StyleManager,
    }

    impl Default for ManuscriptThemeSwitcher {
        fn default() -> Self {
            Self {
                system_selector: TemplateChild::default(),
                light_selector: TemplateChild::default(),
                sepia_selector: TemplateChild::default(),
                dark_selector: TemplateChild::default(),
                settings: ManuscriptSettings::default(),
                style_manager: adw::StyleManager::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptThemeSwitcher {
        const NAME: &'static str = "ManuscriptThemeSwitcher";
        type Type = super::ManuscriptThemeSwitcher;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptThemeSwitcher {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_active_switch(Theme::current());
            self.obj().connect_events();
        }
    }
    impl BoxImpl for ManuscriptThemeSwitcher {}
    impl WidgetImpl for ManuscriptThemeSwitcher {}
}

glib::wrapper! {
    pub struct ManuscriptThemeSwitcher(ObjectSubclass<imp::ManuscriptThemeSwitcher>)
        @extends gtk::Box, gtk::Widget, @implements gio::ActionGroup, gio::ActionMap, gtk::Buildable;
}

impl Default for ManuscriptThemeSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ManuscriptThemeSwitcher {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn connect_events(&self) {
        self.imp().settings.connect_changed(
            "color-scheme",
            glib::clone!(@weak self as this => move |_, _| {
                this.set_active_switch(this.imp().settings.color_scheme());
            }),
        );
    }

    pub fn set_active_switch(&self, theme: String) {
        let imp = self.imp();

        match theme.as_str() {
            "system" => {
                imp.system_selector.set_active(true);
                imp.style_manager
                    .set_color_scheme(adw::ColorScheme::PreferLight);
            }
            "light" => {
                imp.light_selector.set_active(true);
                imp.style_manager
                    .set_color_scheme(adw::ColorScheme::ForceLight);
            }
            "sepia" => {
                imp.sepia_selector.set_active(true);
                imp.style_manager
                    .set_color_scheme(adw::ColorScheme::ForceLight);
            }
            "dark" => {
                imp.dark_selector.set_active(true);
                imp.style_manager
                    .set_color_scheme(adw::ColorScheme::ForceDark);
            }
            _ => unreachable!("Theme not supported"),
        }
    }

    pub fn set_selected_theme(&self, theme: String) {
        Theme::set_current(theme);
    }
}

#[gtk::template_callbacks]
impl ManuscriptThemeSwitcher {
    #[template_callback]
    fn on_color_scheme_changed(&self, _param: glib::ParamSpec, _button: &gtk::CheckButton) {
        let imp = self.imp();
        if imp.system_selector.is_active() {
            self.set_selected_theme("system".into());
        } else if imp.light_selector.is_active() {
            self.set_selected_theme("light".into());
        } else if imp.sepia_selector.is_active() {
            self.set_selected_theme("sepia".into());
        } else if imp.dark_selector.is_active() {
            self.set_selected_theme("dark".into());
        }
    }
}
