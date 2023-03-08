use crate::{models::*, services::i18n::i18n};
use adw::prelude::{ActionRowExt, PreferencesRowExt};
use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};
use std::cell::RefCell;

#[allow(unused)]
const G_LOG_DOMAIN: &str = "ManuscriptChunkRow";

mod imp {
    use super::*;
    use glib::ParamSpec;
    use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/chunk_row.ui")]
    pub struct ManuscriptChunkRow {
        #[template_child]
        pub(super) lock_icon: TemplateChild<gtk::Image>,

        pub(super) chunk_id: RefCell<String>,
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
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(Vec::new);
            PROPERTIES.as_ref()
        }

        // fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
        //     let _obj = self.obj();
        //     match pspec.name() {
        //         _ => unimplemented!(),
        //     }
        // }

        fn set_property(&self, _id: usize, _value: &glib::Value, _pspec: &ParamSpec) {
            // let _obj = self.obj();
            // match pspec.name() {
            //     _ => unimplemented!(),
            // }
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.obj().connect_events();
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

impl Default for ManuscriptChunkRow {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ManuscriptChunkRow {
    pub fn new(chunk: Option<&dyn DocumentChunk>) -> Self {
        let obj: Self = glib::Object::new(&[]);
        obj.setup(chunk);
        obj
    }

    pub fn setup(&self, chunk: Option<&dyn DocumentChunk>) {
        if let Some(chunk) = chunk {
            *self.imp().chunk_id.borrow_mut() = chunk.id().to_string();
            self.set_title(chunk.safe_title());
            self.lock_icon().set_visible(chunk.locked());
            if let Some(chapter) = chunk.as_any().downcast_ref::<Chapter>() {
                self.set_subtitle(format!("{} {}", chapter.words_count(), i18n("words")).as_str());
            } else if let Some(character_sheet) = chunk.as_any().downcast_ref::<CharacterSheet>() {
                self.set_subtitle(character_sheet.role().unwrap_or(&i18n("No role")).as_str());
            }
        } else {
            *self.imp().chunk_id.borrow_mut() = "".into();
            self.set_title("");
            self.set_subtitle("");
            self.lock_icon().set_visible(false);
        }
    }

    fn connect_events(&self) {}

    fn lock_icon(&self) -> gtk::Image {
        self.imp().lock_icon.get()
    }

    pub fn chunk_id(&self) -> String {
        self.imp().chunk_id.borrow().clone()
    }
}
