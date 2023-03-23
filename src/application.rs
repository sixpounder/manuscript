use crate::{
    config::{APPLICATION_G_PATH, VERSION},
    services::{i18n::translators_list, ManuscriptSettings},
    widgets::Theme,
    ManuscriptWindow,
};
use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ManuscriptApplication {
        pub(super) settings: ManuscriptSettings,
        pub(super) sepia_style_provider: RefCell<Option<gtk::CssProvider>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptApplication {
        const NAME: &'static str = "ManuscriptApplication";
        type Type = super::ManuscriptApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for ManuscriptApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.instance();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.set_accels_for_action("app.new-window", &["<ctrl><shift>n"]);
            obj.set_accels_for_action("app.new-project", &["<ctrl>n"]);
            obj.set_accels_for_action("app.open-project", &["<ctrl>o"]);
            obj.set_accels_for_action("project.save", &["<ctrl>s"]);
            obj.set_accels_for_action("win.toggle-command-palette", &["<ctrl><shift>p"]);
            obj.set_accels_for_action("project.search", &["<ctrl>f"]);
        }
    }

    impl ApplicationImpl for ManuscriptApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.instance();
            application.set_color_scheme(Theme::current().as_str());
            // Get the current window or create one if necessary
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = ManuscriptWindow::new(&*application);
                window.upcast()
            };

            // Ask the window manager/compositor to present the window
            window.present();
        }

        fn startup(&self) {
            self.parent_startup();
            let style_manager = adw::StyleManager::default();
            let obj = self.obj();
            style_manager.connect_dark_notify(glib::clone!(@strong obj as this => move |_| {
                this.set_color_scheme(Theme::current().as_str());
            }));

            let settings = &self.settings;
            settings.connect_changed(
                "color-scheme",
                glib::clone!(@strong obj as this => move |_, _key| {
                    let theme = Theme::current();
                    this.set_color_scheme(theme.as_str());
                }),
            );
            let sepia_style_provider = gtk::CssProvider::default();
            sepia_style_provider.load_from_resource(
                format!("{}/{}", APPLICATION_G_PATH, "style-sepia.css").as_str(),
            );
            *self.sepia_style_provider.borrow_mut() = Some(sepia_style_provider);
        }
    }

    impl GtkApplicationImpl for ManuscriptApplication {}
    impl AdwApplicationImpl for ManuscriptApplication {}
}

glib::wrapper! {
    pub struct ManuscriptApplication(ObjectSubclass<imp::ManuscriptApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::new(&[("application-id", &application_id), ("flags", flags)])
    }

    fn set_color_scheme(&self, scheme: &str) {
        glib::g_debug!(
            "ManuscriptApplication",
            "Setting color scheme to {}",
            scheme
        );
        let display = gtk::gdk::Display::default().unwrap();
        let sepia_provider = self.imp().sepia_style_provider.borrow();
        let sepia_provider: &gtk::CssProvider = sepia_provider.as_ref().unwrap();
        if scheme == "sepia" {
            gtk::StyleContext::add_provider_for_display(
                &display,
                sepia_provider,
                gtk::STYLE_PROVIDER_PRIORITY_USER,
            );
        } else {
            gtk::StyleContext::remove_provider_for_display(&display, sepia_provider);
        }
    }

    fn setup_gactions(&self) {
        let new_window_action = gio::ActionEntry::builder("new-window")
            .activate(move |application: &Self, _, _| {
                let window = ManuscriptWindow::new(&*application);
                let window = window.upcast::<adw::ApplicationWindow>();

                // Ask the window manager/compositor to present the window
                window.present();
            })
            .build();
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([new_window_action, quit_action, about_action])
            .unwrap();
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutWindow::builder()
            .transient_for(&window)
            .application_name("Manuscript")
            .application_icon("io.sixpounder.Manuscript")
            .developer_name("Andrea Coronese")
            .version(VERSION)
            .website("https://github.com/sixpounder/manuscript")
            .issue_url("https://github.com/sixpounder/manuscript/issues")
            .developers(vec!["Andrea Coronese".into()])
            .translator_credits(translators_list().join("\n").as_str())
            .copyright("© 2023 Andrea Coronese")
            .build();

        about.present();
    }
}
