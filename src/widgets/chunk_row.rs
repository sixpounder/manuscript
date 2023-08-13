use crate::{models::*, services::i18n::i18n, widgets::ManuscriptProjectLayoutChunkContainer};
use adw::prelude::{ActionRowExt, PreferencesRowExt};
use adw::subclass::prelude::*;
use glib_macros::Properties;
use gtk::{gio, glib, prelude::*};
use std::cell::{Cell, RefCell};

#[allow(unused)]
const G_LOG_DOMAIN: &str = "ManuscriptChunkRow";

mod imp {
    use super::*;
    use glib::ParamSpec;

    /// A widget to display as a row entry in the project layout in place of a
    /// project chunk
    #[derive(Default, Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::ManuscriptChunkRow)]
    #[template(resource = "/io/sixpounder/Manuscript/chunk_row.ui")]
    pub struct ManuscriptChunkRow {
        #[template_child]
        pub(super) lock_icon: TemplateChild<gtk::Image>,

        #[property(name = "parent-container", get, set, nullable)]
        pub(super) parent_container: RefCell<Option<ManuscriptProjectLayoutChunkContainer>>,

        pub(super) chunk_id: RefCell<String>,

        #[property(name = "select-mode", get, set = Self::set_select_mode)]
        pub(super) select_mode: Cell<bool>,

        #[property(name = "locked", get, set)]
        pub(super) locked: Cell<bool>,

        #[property(name = "selected", get, set)]
        pub(super) selected: Cell<bool>,

        #[property(name = "priority", get, set)]
        pub(super) priority: Cell<u64>,

        #[property(name = "accent", get, set)]
        pub(super) accent: RefCell<Option<Color>>,

        pub(super) style_provider: gtk::CssProvider,
    }

    impl ManuscriptChunkRow {
        pub fn set_select_mode(&self, value: bool) {
            if !value {
                self.obj().set_property("selected", false);
            }

            self.select_mode.set(value)
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptChunkRow {
        const NAME: &'static str = "ManuscriptChunkRow";
        type Type = super::ManuscriptChunkRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptChunkRow {
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

    impl WidgetImpl for ManuscriptChunkRow {}
    impl ListBoxRowImpl for ManuscriptChunkRow {}
    impl PreferencesRowImpl for ManuscriptChunkRow {}
    impl ActionRowImpl for ManuscriptChunkRow {}
}

glib::wrapper! {
    pub struct ManuscriptChunkRow(ObjectSubclass<imp::ManuscriptChunkRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow, @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptChunkRow {
    pub fn new(
        chunk: Option<&dyn DocumentChunk>,
        parent: &ManuscriptProjectLayoutChunkContainer,
    ) -> Self {
        let obj: Self = glib::Object::builder()
            .property("parent-container", parent)
            .property("select-mode", false)
            .build();
        obj.init(chunk);
        obj
    }

    fn init(&self, chunk: Option<&dyn DocumentChunk>) {
        self.update_chunk(chunk);
        if let Some(chunk) = chunk {
            if let Some(chapter) = chunk.as_any().downcast_ref::<Chapter>() {
                self.update_chunk_reading_stats(chapter, chapter.words_count());
            }
        }
    }

    pub fn update_chunk(&self, chunk: Option<&dyn DocumentChunk>) {
        if let Ok(mut borrow) = self.imp().chunk_id.try_borrow_mut() {
            if let Some(chunk) = chunk {
                *borrow = chunk.id().to_string();
                self.set_priority(chunk.priority().unwrap_or(0));
                self.set_title(chunk.safe_title().as_str());
                self.set_locked(chunk.locked());
                self.set_tint(chunk.accent());

                if let Some(character_sheet) = chunk.as_any().downcast_ref::<CharacterSheet>() {
                    self.set_subtitle(character_sheet.role().unwrap_or(&i18n("No role")).as_str());
                }
            } else {
                *borrow = "".into();
                self.set_title("");
                self.set_subtitle("");
                self.lock_icon().set_visible(false);
            }
        } else {
            glib::g_warning!(G_LOG_DOMAIN, "Could not borrow chunk_id cell");
        }
    }

    pub fn set_tint(&self, color: Option<Color>) {
        let tint_provider = &self.imp().style_provider;
        let ctx = self.style_context();
        if let Some(color) = color {
            let mut css = String::new();
            css.push_str(&format!(
                ".chunk-row {{ background-color: {}; color: {}; }}",
                color,
                color.contrast_color()
            ));
            tint_provider.load_from_data(css.as_str());
            ctx.add_provider(tint_provider, gtk::STYLE_PROVIDER_PRIORITY_USER);
        } else {
            ctx.remove_provider(tint_provider);
        }
    }

    pub fn update_chunk_reading_stats(&self, chunk: &dyn DocumentChunk, words_count: u64) {
        if chunk.as_any().downcast_ref::<Chapter>().is_some() {
            self.set_subtitle(format!("{} {}", words_count, i18n("words")).as_str());
        }
    }

    fn lock_icon(&self) -> gtk::Image {
        self.imp().lock_icon.get()
    }

    pub fn chunk_id(&self) -> String {
        self.imp().chunk_id.borrow().clone()
    }
}
