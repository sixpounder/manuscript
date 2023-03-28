use super::factories;
use crate::{
    models::*,
    services::{i18n, DocumentAction},
    widgets::{
        dialogs::ManuscriptEntryInputDialog, ManuscriptChunkRow, ManuscriptPrimaryMenuButton,
    },
};
use adw::{
    prelude::{ActionRowExt, ExpanderRowExt, MessageDialogExt},
    subclass::prelude::*,
};
use glib::Sender;
use gtk::{gio, prelude::*};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

const G_LOG_DOMAIN: &str = "ManuscriptProjectLayout";

mod imp {
    use super::*;
    use glib::{subclass::signal::Signal, ParamSpec, ParamSpecBoolean, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/project_layout.ui")]
    pub struct ManuscriptProjectLayout {
        #[template_child]
        pub(super) header_bar: TemplateChild<adw::HeaderBar>,

        #[template_child]
        pub(super) primary_menu_button: TemplateChild<ManuscriptPrimaryMenuButton>,

        #[template_child]
        pub(super) layout: TemplateChild<gtk::Box>,

        #[template_child]
        pub(super) searchbar: TemplateChild<gtk::SearchBar>,

        #[template_child]
        pub(super) searchentry: TemplateChild<gtk::SearchEntry>,

        #[template_child]
        pub(super) project_actionbar: TemplateChild<gtk::ActionBar>,

        pub(super) title: RefCell<String>,

        pub(super) channel: RefCell<Option<Sender<DocumentAction>>>,

        pub(super) children_map: RefCell<HashMap<String, ManuscriptChunkRow>>,

        pub(super) chunk_selection: Cell<bool>,

        pub(super) selected_ids: RefCell<Vec<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptProjectLayout {
        const NAME: &'static str = "ManuscriptProjectLayout";
        type Type = super::ManuscriptProjectLayout;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptProjectLayout {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_widgets();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("remove-selected-activated")
                    .param_types([Vec::<String>::static_type()])
                    .build()]
            });

            SIGNALS.as_ref()
        }

        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("title")
                        .default_value(None)
                        .readwrite()
                        .build(),
                    ParamSpecString::builder("selection-label")
                        .default_value(Some(""))
                        .read_only()
                        .build(),
                    ParamSpecBoolean::builder("select-mode")
                        .read_only()
                        .default_value(false)
                        .build(),
                    ParamSpecBoolean::builder("show-primary-menu-button")
                        .readwrite()
                        .default_value(false)
                        .build(),
                    ParamSpecBoolean::builder("show-end-title-buttons")
                        .default_value(false)
                        .readwrite()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => self.obj().document_title_label_text().to_value(),
                "select-mode" => self.chunk_selection.get().to_value(),
                "selection-label" => {
                    let items_len = self.selected_ids.borrow().len();
                    if items_len == 0 {
                        i18n::i18n("No items selected").to_value()
                    } else {
                        format!(
                            "{} {}",
                            items_len,
                            i18n::ni18n("item selected", "items selected", items_len as u32)
                        )
                        .to_value()
                    }
                }
                "show-primary-menu-button" => self.primary_menu_button.is_visible().to_value(),
                "show-end-title-buttons" => self.obj().show_end_title_buttons().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "title" => {
                    obj.set_document_title_label_text(value.get::<Option<String>>().unwrap())
                }
                "show-primary-menu-button" => self
                    .primary_menu_button
                    .set_visible(value.get::<bool>().unwrap()),
                "show-end-title-buttons" => {
                    obj.set_show_end_title_buttons(value.get::<bool>().unwrap())
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ManuscriptProjectLayout {}
}

glib::wrapper! {
    pub struct ManuscriptProjectLayout(ObjectSubclass<imp::ManuscriptProjectLayout>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ManuscriptProjectLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl ManuscriptProjectLayout {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_channel(&self, sender: Sender<DocumentAction>) {
        self.imp().channel.replace(Some(sender));
    }

    fn setup_widgets(&self) {
        self.style_context().add_class("default-bg");

        let entry = self.imp().searchentry.get();
        self.imp().searchbar.connect_entry(&entry);
    }

    pub fn document_title_label_text(&self) -> String {
        self.imp().title.borrow().clone()
    }

    pub fn set_document_title_label_text(&self, value: Option<String>) {
        let new_title = value.unwrap_or(i18n::i18n("Untitled Project"));
        *self.imp().title.borrow_mut() = new_title.clone();
        self.notify("title");
    }

    pub fn load_document(&self, document: Option<&Document>) {
        glib::debug!("Loading document on project layout widget");
        // Clear layout
        self.clear();

        if let Some(document) = document {
            // Populate project structure
            // 1. Set title
            self.set_document_title_label_text(document.title().cloned());

            // 2. Add chunk entries
            document.chunks().iter().for_each(|chunk| {
                self.add_chunk(*chunk);
            });
        }
    }

    pub fn clear(&self) {
        let layout = self.imp().layout.get();
        let mut child = layout.first_child();
        while child.is_some() {
            let existing_child = child.unwrap();
            child = existing_child.next_sibling();
            layout.remove(&existing_child);
        }
        self.set_select(false);
    }

    fn expanders(&self) -> Vec<gtk::Widget> {
        let layout = self.imp().layout.get();
        let mut children = vec![];
        let mut child = layout.first_child();
        while child.is_some() {
            let existing_child = child.unwrap();
            child = existing_child.next_sibling();
            children.push(existing_child);
        }
        children
    }

    pub fn add_chunk(&self, chunk: &dyn DocumentChunk) {
        glib::g_debug!(
            G_LOG_DOMAIN,
            "Adding chunk with id {} to project layout",
            chunk.id()
        );
        let layout = self.imp().layout.get();
        let expander_row =
            factories::get_or_create_expander_row_for_chunk(&layout.upcast::<gtk::Widget>(), chunk);
        expander_row.set_expanded(true);

        let row = factories::create_row_for_chunk(chunk, expander_row.clone());
        row.connect_activated(glib::clone!(@weak self as this => move |row| {
            if row.select_mode() {
                let selected = !row.selected();
                row.set_property("selected", selected);
            } else {
                this.send_action(DocumentAction::SelectChunk(row.chunk_id()));
            }
        }));

        row.connect_notify_local(
            Some("selected"),
            glib::clone!(@weak self as this => move |row, _| {
                {
                    let mut selected_ids = this.imp().selected_ids.borrow_mut();
                    if row.selected() {
                        selected_ids.push(row.chunk_id());
                    } else if let Some(index) = selected_ids.iter().position(|entry| entry.as_str() == row.chunk_id().as_str()) {
                        selected_ids.remove(index);
                    }
                }
                this.notify("selection-label");
            })
        );

        let mut map = self.imp().children_map.borrow_mut();
        map.insert(chunk.id().to_string(), row.clone());

        expander_row.add_row(&row);
    }

    pub fn remove_chunk<S: ToString>(&self, chunk_id: S) {
        let next_select_mode;

        {
            glib::g_debug!(
                G_LOG_DOMAIN,
                "Removing chunk with id {} from project layout",
                chunk_id.to_string()
            );

            let mut map = self.imp().children_map.borrow_mut();
            if let Some(removed) = map.remove(&chunk_id.to_string()) {
                if let Some(parent) = removed.parent_expander() {
                    parent.remove(&removed);
                }
            }

            glib::g_debug!(
                G_LOG_DOMAIN,
                "Removed chunk with id {} from project layout",
                chunk_id.to_string()
            );

            next_select_mode = !map.is_empty();
        }

        self.set_select(next_select_mode);
    }

    pub fn chunk_row(&self, chunk: &dyn DocumentChunk) -> Option<ManuscriptChunkRow> {
        let widget_ref = self.imp().children_map.borrow();
        widget_ref.get(&chunk.id().to_string()).cloned()
    }

    pub fn searchbar(&self) -> gtk::SearchBar {
        self.imp().searchbar.get()
    }

    pub fn set_search(&self, value: bool) {
        self.searchbar().set_search_mode(value);
    }

    pub fn set_select(&self, value: bool) {
        let imp = self.imp();
        self.clear_all_rows();

        if value != imp.chunk_selection.replace(value) {
            imp.project_actionbar.set_revealed(value);
            let children = imp.children_map.borrow();
            children
                .iter()
                .for_each(|(_key, child)| child.set_property("select-mode", value));

            if value {
                self.expanders().iter().for_each(|expander| {
                    if let Some(expander) = expander.downcast_ref::<adw::ExpanderRow>() {
                        expander.set_expanded(true);
                    }
                });
            }

            self.notify("selection-label");
            self.notify("select-mode");
        }
    }

    fn select_all_rows(&self) {
        self.clear_all_rows();
        let children = self.imp().children_map.borrow();
        children
            .iter()
            .for_each(|(_key, child)| child.set_property("selected", true));
    }

    fn clear_all_rows(&self) {
        self.imp().selected_ids.borrow_mut().clear();
    }

    fn send_action(&self, action: DocumentAction) {
        let maybe_channel = self.imp().channel.borrow();
        if let Some(channel) = maybe_channel.as_ref() {
            channel.send(action).expect("Could not send action");
        }
    }

    pub fn show_end_title_buttons(&self) -> bool {
        self.imp().header_bar.shows_end_title_buttons()
    }

    pub fn set_show_end_title_buttons(&self, value: bool) {
        self.imp().header_bar.set_show_end_title_buttons(value);
        self.notify("show-end-title-buttons");
    }
}

#[gtk::template_callbacks]
impl ManuscriptProjectLayout {
    #[template_callback]
    fn on_title_clicked(&self, _button: &gtk::Button) {
        let current_title: String = self.document_title_label_text();
        let dialog = ManuscriptEntryInputDialog::new(
            &crate::libs::files::window(),
            &[
                ("heading", &i18n::i18n("Change Project Title")),
                (
                    "body",
                    &i18n::i18n("This will appear in the cover page of your exports"),
                ),
                ("entry-text", &current_title),
                ("entry-label", &i18n::i18n("Project Title")),
            ],
        );
        dialog.connect_response(
            None,
            glib::clone!(@strong self as this => move |dialog, res| {
                if res == "confirm" {
                    this.send_action(DocumentAction::SetTitle(dialog.entry_text()));
                }
            }),
        );
        dialog.show();
    }

    #[template_callback]
    fn on_remove_items_clicked(&self, _button: &gtk::Button) {
        let borrow = self.imp().selected_ids.borrow();
        let ids = borrow.clone();
        drop(borrow);
        self.emit_by_name::<()>("remove-selected-activated", &[&ids]);
    }

    #[template_callback]
    fn on_select_all_button_clicked(&self, _button: &gtk::Button) {
        self.select_all_rows();
    }
}
