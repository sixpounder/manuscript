use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::{config::VERSION, services::i18n::translators_list, ManuscriptWindow};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ManuscriptApplication {}

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

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action])
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
