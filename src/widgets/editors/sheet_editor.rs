use crate::{models::*, services::prelude::*, services::*};
use adw::subclass::prelude::*;
use bytes::Bytes;
use gtk::{gio, glib, glib::Sender, prelude::*};
use std::cell::{Cell, Ref, RefCell};

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

        pub(super) chunk_id: RefCell<String>,
        pub(super) character_name: RefCell<Option<String>>,
        pub(super) character_role: RefCell<Option<String>>,
        pub(super) character_gender: Cell<Gender>,
        pub(super) character_background_buffer: RefCell<Option<gtk::TextBuffer>>,
        pub(super) character_physical_traits_buffer: RefCell<Option<gtk::TextBuffer>>,
        pub(super) character_psycological_traits_buffer: RefCell<Option<gtk::TextBuffer>>,
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
            self.obj().setup_widgets();
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
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        "character-physical-traits-buffer",
                        "",
                        "",
                        None,
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecString::new(
                        "character-psycological-traits-buffer",
                        "",
                        "",
                        None,
                        ParamFlags::READWRITE,
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
                "character-background-buffer" => obj.set_character_background_buffer(
                    value.get::<Option<gtk::TextBuffer>>().expect("Wrong value"),
                ),
                "character-physical-traits-buffer" => obj.set_character_physical_traits_buffer(
                    value.get::<Option<gtk::TextBuffer>>().expect("Wrong value"),
                ),
                "character-psycological-traits-buffer" => obj
                    .set_character_psycological_traits_buffer(
                        value.get::<Option<gtk::TextBuffer>>().expect("Wrong value"),
                    ),
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
    pub fn new(chunk_id: String, sender: Option<Sender<DocumentAction>>) -> Self {
        let obj: Self = glib::Object::new(&[]);
        obj.set_chunk_id(chunk_id);
        obj.set_sender(sender);

        obj
    }

    fn set_chunk_id(&self, value: String) {
        *self.imp().chunk_id.borrow_mut() = value;
    }

    fn set_sender(&self, value: Option<Sender<DocumentAction>>) {
        *self.imp().sender.borrow_mut() = value;
    }

    pub fn character_name(&self) -> Option<String> {
        self.imp().character_name.borrow().clone()
    }

    pub fn set_character_name(&self, value: Option<String>) {
        let update_value = value.clone();
        *self.imp().character_name.borrow_mut() = value;
        self.send_update(move |chunk| {
            let obj = chunk
                .as_any_mut()
                .downcast_mut::<CharacterSheet>()
                .expect("How?");
            obj.set_name(update_value);
        });
    }

    pub fn character_gender(&self) -> Gender {
        self.imp().character_gender.get()
    }

    pub fn set_character_gender(&self, value: Gender) {
        self.imp().character_gender.set(value);
    }

    pub fn character_role(&self) -> Option<String> {
        self.imp().character_role.borrow().clone()
    }

    pub fn set_character_role(&self, value: Option<String>) {
        self.imp().character_name.replace(value);
    }

    pub fn character_background(&self) -> Option<Bytes> {
        self.character_background_buffer().as_ref().map(bytes_from_text_buffer)
    }

    pub fn set_character_background(&self, value: Option<Bytes>) {
        let update_value = value.clone();
        let text_buffer = value.map(bytes_to_text_buffer);
        self.set_character_background_buffer(text_buffer);

        self.send_update(move |chunk| {
            let obj = chunk
                .as_any_mut()
                .downcast_mut::<CharacterSheet>()
                .expect("How?");
            obj.set_background_bytes(update_value.unwrap_or_default());
        });
    }

    pub fn character_physical_traits(&self) -> Option<Bytes> {
        self.character_physical_traits_buffer().as_ref().map(bytes_from_text_buffer)
    }

    pub fn set_character_physical_traits(&self, value: Option<Bytes>) {
        self.set_character_physical_traits_buffer(value.map(bytes_to_text_buffer));
        self.notify("character-physical-traits-buffer");
    }

    pub fn character_psycological_traits(&self) -> Option<Bytes> {
        self.character_psycological_traits_buffer().as_ref().map(bytes_from_text_buffer)
    }

    pub fn set_character_psycological_traits(&self, value: Option<Bytes>) {
        self.set_character_psycological_traits_buffer(value.map(bytes_to_text_buffer));
        self.notify("character-psycological-traits-buffer");
    }

    fn character_background_buffer(&self) -> Ref<Option<gtk::TextBuffer>> {
        self.imp().character_background_buffer.borrow()
    }

    fn set_character_background_buffer(&self, value: Option<gtk::TextBuffer>) {
        self.imp().character_background_buffer.replace(value);
    }

    fn character_physical_traits_buffer(&self) -> Ref<Option<gtk::TextBuffer>> {
        self.imp().character_physical_traits_buffer.borrow()
    }

    fn set_character_physical_traits_buffer(&self, value: Option<gtk::TextBuffer>) {
        self.imp().character_physical_traits_buffer.replace(value);
    }

    fn character_psycological_traits_buffer(&self) -> Ref<Option<gtk::TextBuffer>> {
        self.imp().character_psycological_traits_buffer.borrow()
    }

    fn set_character_psycological_traits_buffer(&self, value: Option<gtk::TextBuffer>) {
        self.imp()
            .character_psycological_traits_buffer
            .replace(value);
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

    fn setup_widgets(&self) {
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

        if let Some(character_background_buffer) = imp.character_background_buffer.borrow().as_ref() {
            character_background_buffer
                .connect_changed(glib::clone!(@weak self as this => move |buf| {
                    this.set_character_background(Some(bytes_from_text_buffer(buf)))
                }));
        }

        if let Some(character_physical_traits_buffer) = imp.character_physical_traits_buffer.borrow().as_ref() {
            character_physical_traits_buffer
                .connect_changed(glib::clone!(@weak self as this => move |buf| {
                    this.set_character_physical_traits(Some(bytes_from_text_buffer(buf)))
                }));
        }

        if let Some(character_psycological_traits_buffer) = imp.character_psycological_traits_buffer.borrow().as_ref() {
            character_psycological_traits_buffer
                .connect_changed(glib::clone!(@weak self as this => move |buf| {
                    this.set_character_psycological_traits(Some(bytes_from_text_buffer(buf)))
                }));
        }
    }
}
