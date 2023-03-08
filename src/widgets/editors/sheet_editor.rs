use crate::{models::*, services::prelude::*, services::*};
use adw::{prelude::*, subclass::prelude::*};
use bytes::Bytes;
use gtk::{gio, glib, glib::Sender};
use std::cell::RefCell;

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/editors/sheet_editor.ui")]
    pub struct ManuscriptCharacterSheetEditor {
        #[template_child]
        pub(super) character_name_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub(super) character_role_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub(super) character_gender_entry: TemplateChild<adw::ComboRow>,

        #[template_child]
        pub(super) background_text_view: TemplateChild<gtk::TextView>,

        #[template_child]
        pub(super) physical_traits_text_view: TemplateChild<gtk::TextView>,

        #[template_child]
        pub(super) psycological_traits_text_view: TemplateChild<gtk::TextView>,

        #[template_child]
        pub(super) character_background_buffer: TemplateChild<gtk::TextBuffer>,

        #[template_child]
        pub(super) character_physical_traits_buffer: TemplateChild<gtk::TextBuffer>,

        #[template_child]
        pub(super) character_psycological_traits_buffer: TemplateChild<gtk::TextBuffer>,

        pub(super) chunk_id: RefCell<String>,
        pub(super) sender: RefCell<Option<Sender<DocumentAction>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptCharacterSheetEditor {
        const NAME: &'static str = "ManuscriptCharacterSheetEditor";
        type Type = super::ManuscriptCharacterSheetEditor;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptCharacterSheetEditor {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("character-name", "", "", None, ParamFlags::READWRITE),
                    ParamSpecString::new("character-role", "", "", None, ParamFlags::READWRITE),
                    ParamSpecString::new(
                        "character-background-buffer",
                        "",
                        "",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "character-physical-traits-buffer",
                        "",
                        "",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "character-psycological-traits-buffer",
                        "",
                        "",
                        None,
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "character-name" => obj.character_name().to_value(),
                "character-role" => obj.character_role().to_value(),
                "character-background-buffer" => obj.character_background_buffer().to_value(),
                "character-physical-traits-buffer" => {
                    obj.character_physical_traits_buffer().to_value()
                }
                "character-psycological-traits-buffer" => {
                    obj.character_psycological_traits_buffer().to_value()
                }
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "character-name" => {
                    obj.set_character_name(value.get::<Option<String>>().expect("Wrong value"))
                }
                "character-role" => {
                    obj.set_character_role(value.get::<Option<String>>().expect("Wrong value"))
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ManuscriptCharacterSheetEditor {}
}

glib::wrapper! {
    pub struct ManuscriptCharacterSheetEditor(ObjectSubclass<imp::ManuscriptCharacterSheetEditor>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptCharacterSheetEditor {
    pub fn new(source: &dyn DocumentChunk, sender: Option<Sender<DocumentAction>>) -> Self {
        let obj: Self = glib::Object::new(&[]);
        obj.set_chunk_id(source.id().into());
        obj.set_sender(sender);
        if let Some(source) = source.as_any().downcast_ref::<CharacterSheet>() {
            obj.setup_widgets(source);
            obj.connect_events();
        }
        obj
    }

    fn setup_widgets(&self, source: &CharacterSheet) {
        self.background_text_view()
            .set_buffer(Some(&self.character_background_buffer()));
        self.physical_traits_text_view()
            .set_buffer(Some(&self.character_physical_traits_buffer()));
        self.psycological_traits_text_view()
            .set_buffer(Some(&self.character_physical_traits_buffer()));

        self.character_name_entry()
            .set_text(source.name().unwrap_or(&String::default()).as_str());

        self.character_role_entry()
            .set_text(source.role().unwrap_or(&String::default()).as_str());

        self.character_gender_entry()
            .set_selected(source.gender().into());

        self.character_background_buffer().set_text(
            String::from_utf8(source.background().to_vec())
                .unwrap()
                .as_str(),
        );

        self.character_physical_traits_buffer().set_text(
            String::from_utf8(source.physical_traits().to_vec())
                .unwrap()
                .as_str(),
        );

        self.character_psycological_traits_buffer().set_text(
            String::from_utf8(source.psycological_traits().to_vec())
                .unwrap()
                .as_str(),
        );
    }

    fn set_chunk_id(&self, value: String) {
        *self.imp().chunk_id.borrow_mut() = value;
    }

    fn set_sender(&self, value: Option<Sender<DocumentAction>>) {
        *self.imp().sender.borrow_mut() = value;
    }

    pub fn character_name(&self) -> Option<String> {
        Some(self.imp().character_name_entry.text().into())
    }

    pub fn set_character_name(&self, value: Option<String>) {
        self.send_update(move |chunk| {
            let obj = chunk
                .as_any_mut()
                .downcast_mut::<CharacterSheet>()
                .expect("How?");
            obj.set_name(value);
        });
    }

    pub fn character_gender(&self) -> Gender {
        let idx = self.character_gender_entry().selected();
        Gender::from(idx)
    }

    pub fn set_character_gender(&self, value: Gender) {
        self.send_update(move |chunk| {
            let obj = chunk
                .as_any_mut()
                .downcast_mut::<CharacterSheet>()
                .expect("How?");
            obj.set_gender(value);
        });
    }

    pub fn character_role(&self) -> Option<String> {
        Some(self.imp().character_role_entry.text().into())
    }

    pub fn set_character_role(&self, value: Option<String>) {
        self.send_update(move |chunk| {
            let obj = chunk
                .as_any_mut()
                .downcast_mut::<CharacterSheet>()
                .expect("How?");
            obj.set_role(value);
        });
    }

    pub fn character_background(&self) -> Bytes {
        bytes_from_text_buffer(&self.character_background_buffer())
    }

    pub fn set_character_background(&self, value: Option<Bytes>) {
        self.send_update(move |chunk| {
            let obj = chunk
                .as_any_mut()
                .downcast_mut::<CharacterSheet>()
                .expect("How?");
            obj.set_background_bytes(value.unwrap_or_default());
        });
    }

    pub fn character_physical_traits(&self) -> Bytes {
        bytes_from_text_buffer(&self.character_physical_traits_buffer())
    }

    pub fn set_character_physical_traits(&self, value: Option<Bytes>) {
        self.send_update(move |chunk| {
            let obj = chunk
                .as_any_mut()
                .downcast_mut::<CharacterSheet>()
                .expect("How?");
            obj.set_physical_traits_bytes(value.unwrap_or_default());
        });
    }

    pub fn character_psycological_traits(&self) -> Bytes {
        bytes_from_text_buffer(&self.character_psycological_traits_buffer())
    }

    pub fn set_character_psycological_traits(&self, value: Option<Bytes>) {
        let update_value = value.clone();

        self.send_update(move |chunk| {
            let obj = chunk
                .as_any_mut()
                .downcast_mut::<CharacterSheet>()
                .expect("How?");
            obj.set_psycological_traits_bytes(update_value.unwrap_or_default());
        });
    }

    fn character_name_entry(&self) -> adw::EntryRow {
        self.imp().character_name_entry.get()
    }

    fn character_role_entry(&self) -> adw::EntryRow {
        self.imp().character_role_entry.get()
    }

    fn character_gender_entry(&self) -> adw::ComboRow {
        self.imp().character_gender_entry.get()
    }

    fn background_text_view(&self) -> gtk::TextView {
        self.imp().background_text_view.get()
    }

    fn physical_traits_text_view(&self) -> gtk::TextView {
        self.imp().physical_traits_text_view.get()
    }

    fn psycological_traits_text_view(&self) -> gtk::TextView {
        self.imp().psycological_traits_text_view.get()
    }

    fn character_background_buffer(&self) -> gtk::TextBuffer {
        self.imp().character_background_buffer.get()
    }

    fn character_physical_traits_buffer(&self) -> gtk::TextBuffer {
        self.imp().character_physical_traits_buffer.get()
    }

    fn character_psycological_traits_buffer(&self) -> gtk::TextBuffer {
        self.imp().character_psycological_traits_buffer.get()
    }

    fn send_update<F>(&self, f: F)
    where
        F: FnOnce(&mut dyn DocumentChunk) + 'static,
    {
        let imp = self.imp();
        let chunk_id = imp.chunk_id.borrow();
        let tx = imp.sender.borrow();
        let tx = tx.as_ref().expect("No channel sender found");
        tx.send(DocumentAction::UpdateChunkWith(
            chunk_id.clone(),
            Box::new(f),
        ))
        .expect("Failed to send character sheet update");
    }

    fn connect_events(&self) {
        let imp = self.imp();

        imp.character_name_entry
            .connect_changed(glib::clone!(@weak self as this => move |_| {
                let imp = this.imp();
                this.set_character_name(Some(imp.character_name_entry.text().into()))
            }));

        imp.character_role_entry
            .connect_changed(glib::clone!(@weak self as this => move |_| {
                let imp = this.imp();
                this.set_character_role(Some(imp.character_role_entry.text().into()))
            }));

        imp.character_gender_entry.connect_notify_local(
            Some("selected-item"),
            glib::clone!(@weak self as this => move |entry, _idx| {
                let idx = entry.selected();
                let gender = Gender::from(idx);
                this.set_character_gender(gender);
            }),
        );

        self.character_background_buffer().connect_changed(
            glib::clone!(@weak self as this => move |buf| {
                this.set_character_background(Some(bytes_from_text_buffer(buf)))
            }),
        );

        self.character_physical_traits_buffer().connect_changed(
            glib::clone!(@weak self as this => move |buf| {
                this.set_character_physical_traits(Some(bytes_from_text_buffer(buf)))
            }),
        );

        self.character_psycological_traits_buffer().connect_changed(
            glib::clone!(@weak self as this => move |buf| {
                this.set_character_psycological_traits(Some(bytes_from_text_buffer(buf)))
            }),
        );
    }
}
