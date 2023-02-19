use crate::config::{APPLICATION_ID, G_LOG_DOMAIN};
use gtk::gdk;
use gtk::gio::prelude::{SettingsExt, SettingsExtManual};
use gtk::glib::IsA;

#[derive(Debug, Clone)]
pub struct ManuscriptSettings {
    inner: gtk::gio::Settings,
}

impl Default for ManuscriptSettings {
    fn default() -> Self {
        Self {
            inner: gtk::gio::Settings::new(APPLICATION_ID),
        }
    }
}

impl ManuscriptSettings {
    pub fn window_width(&self) -> i32 {
        self.inner.int("window-width")
    }

    pub fn set_window_width(&self, value: i32) {
        self.inner
            .set_int("window-width", value)
            .expect("Could not store window width");
    }

    pub fn window_height(&self) -> i32 {
        self.inner.int("window-height")
    }

    pub fn set_window_height(&self, value: i32) {
        self.inner
            .set_int("window-height", value)
            .expect("Could not store window width");
    }

    pub fn connect_changed<F>(&self, key: &str, f: F)
    where
        F: Fn(&gtk::gio::Settings, &str) + 'static,
    {
        self.inner.connect_changed(Some(key), move |settings, key| {
            glib::info!("GSettings:{} changed", key);
            f(settings, key);
        });
    }

    pub fn bind<P>(&self, key: &str, object: &P, property: &str)
    where
        P: IsA<glib::Object>,
    {
        self.inner.bind(key, object, property).build();
    }
}
