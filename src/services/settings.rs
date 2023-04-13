use crate::config::{APPLICATION_ID, G_LOG_DOMAIN};
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
    pub fn color_scheme(&self) -> String {
        self.inner.string("color-scheme").into()
    }

    pub fn set_color_scheme(&self, value: String) {
        self.inner
            .set_string("color-scheme", value.as_str())
            .expect("Could not store color scheme");
    }

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

    pub fn last_opened_document(&self) -> String {
        self.inner.string("last-opened-document").into()
    }

    pub fn set_last_opened_document(&self, value: &str) {
        self.inner
            .set_string("last-opened-document", value)
            .expect("Could not store last opened document");
    }

    pub fn text_analysis_delay(&self) -> i32 {
        self.inner.int("text-analysis-delay")
    }

    pub fn set_text_analysis_delay(&self, value: i32) {
        self.inner
            .set_int("text-analysis-delay", value)
            .expect("Could not store text analysis delay");
    }

    pub fn chunk_props_panel_visible(&self) -> bool {
        self.inner.boolean("chunk-props-panel-visible")
    }

    pub fn set_chunk_props_panel_visible(&self, value: bool) {
        self.inner
            .set_boolean("chunk-props-panel-visible", value)
            .expect("Could not store chunk-props-panel-visible");
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
