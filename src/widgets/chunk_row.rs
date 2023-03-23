use crate::{models::*, services::i18n::i18n};
use adw::prelude::{ActionRowExt, PreferencesRowExt};
use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};
use std::cell::{Cell, RefCell};

#[allow(unused)]
const G_LOG_DOMAIN: &str = "ManuscriptChunkRow";

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject};
    use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/chunk_row.ui")]
    pub struct ManuscriptChunkRow {
        #[template_child]
        pub(super) lock_icon: TemplateChild<gtk::Image>,

        pub(super) parent_expander: RefCell<Option<adw::ExpanderRow>>,

        pub(super) chunk_id: RefCell<String>,

        pub(super) select_mode: Cell<bool>,

        pub(super) locked: Cell<bool>,

        pub(super) selected: Cell<bool>,

        pub(super) style_provider: gtk::CssProvider,
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
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "expander",
                        "",
                        "",
                        Option::<adw::ExpanderRow>::static_type(),
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new("locked", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("selected", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("select-mode", "", "", false, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "expander" => self.parent_expander.borrow().to_value(),
                "locked" => obj.locked().to_value(),
                "selected" => obj.selected().to_value(),
                "select-mode" => obj.select_mode().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "expander" => {
                    *self.parent_expander.borrow_mut() =
                        value.get::<Option<adw::ExpanderRow>>().unwrap()
                }
                "locked" => obj.set_locked(value.get::<bool>().unwrap()),
                "selected" => obj.set_selected(value.get::<bool>().unwrap()),
                "select-mode" => obj.set_select_mode(value.get::<bool>().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
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
    pub fn new(chunk: Option<&dyn DocumentChunk>, expander: adw::ExpanderRow) -> Self {
        let obj: Self = glib::Object::new(&[("expander", &expander), ("select-mode", &false)]);
        obj.update_chunk(chunk);
        obj
    }

    pub fn selected(&self) -> bool {
        self.imp().selected.get()
    }

    pub fn set_selected(&self, value: bool) {
        self.imp().selected.set(value)
    }

    pub fn locked(&self) -> bool {
        self.imp().locked.get()
    }

    pub fn set_locked(&self, value: bool) {
        self.imp().locked.set(value)
    }

    pub fn select_mode(&self) -> bool {
        self.imp().select_mode.get()
    }

    pub fn set_select_mode(&self, value: bool) {
        if !value {
            self.set_property("selected", false);
        }

        self.imp().select_mode.set(value)
    }

    pub fn update_chunk(&self, chunk: Option<&dyn DocumentChunk>) {
        if let Ok(mut borrow) = self.imp().chunk_id.try_borrow_mut() {
            if let Some(chunk) = chunk {
                *borrow = chunk.id().to_string();
                self.set_title(chunk.safe_title());
                self.set_locked(chunk.locked());
                self.set_tint(chunk.accent());

                if let Some(chapter) = chunk.as_any().downcast_ref::<Chapter>() {
                    self.set_subtitle(
                        format!("{} {}", chapter.words_count(), i18n("words")).as_str(),
                    );
                } else if let Some(character_sheet) =
                    chunk.as_any().downcast_ref::<CharacterSheet>()
                {
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
                ".chunk-row {{ background-color: {}; color: {} }}",
                color,
                color.contrast_color()
            ));
            tint_provider.load_from_data(css.as_bytes());
            ctx.add_provider(tint_provider, gtk::STYLE_PROVIDER_PRIORITY_USER);
        } else {
            ctx.remove_provider(tint_provider);
        }
    }

    pub fn update_chunk_reading_stats(&self, chunk: Option<&dyn DocumentChunk>, words_count: u64) {
        if let Some(chunk) = chunk {
            if chunk.as_any().downcast_ref::<Chapter>().is_some() {
                self.set_subtitle(format!("{} {}", words_count, i18n("words")).as_str());
            }
        }
    }

    fn lock_icon(&self) -> gtk::Image {
        self.imp().lock_icon.get()
    }

    pub fn chunk_id(&self) -> String {
        self.imp().chunk_id.borrow().clone()
    }

    pub fn parent_expander(&self) -> Option<adw::ExpanderRow> {
        self.imp().parent_expander.borrow().clone()
    }
}
