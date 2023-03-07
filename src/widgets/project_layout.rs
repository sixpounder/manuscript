use super::factories;
use crate::{models::*, services::DocumentAction};
use adw::{
    prelude::{ActionRowExt, ExpanderRowExt},
    subclass::prelude::*,
};
use glib::Sender;
use gtk::{gio, prelude::*};
use std::cell::RefCell;

const G_LOG_DOMAIN: &str = "ManuscriptProjectLayout";

mod imp {
    use super::*;
    use glib::ParamSpec;
    use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/project_layout.ui")]
    pub struct ManuscriptProjectLayout {
        #[template_child]
        pub(super) layout: TemplateChild<gtk::Box>,

        #[template_child]
        pub(super) title_entry: TemplateChild<gtk::Entry>,

        #[template_child]
        pub(super) title_popover: TemplateChild<gtk::Popover>,

        #[template_child]
        pub(super) searchbar: TemplateChild<gtk::SearchBar>,

        #[template_child]
        pub searchentry: TemplateChild<gtk::SearchEntry>,

        pub(super) channel: RefCell<Option<Sender<DocumentAction>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptProjectLayout {
        const NAME: &'static str = "ManuscriptProjectLayout";
        type Type = super::ManuscriptProjectLayout;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
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
        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(Vec::new);
            PROPERTIES.as_ref()
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
        glib::Object::new(&[])
    }

    pub fn set_channel(&self, sender: Sender<DocumentAction>) {
        self.imp().channel.replace(Some(sender));
    }

    fn setup_widgets(&self) {
        self.style_context().add_class("default-bg");

        let entry = self.imp().searchentry.get();
        self.imp().searchbar.connect_entry(&entry);

        self.imp().title_entry.connect_changed(glib::clone!(@strong self as this => move |entry| {
            let sender = this.imp().channel.borrow();
            let sender = sender.as_ref().unwrap();
            sender.send(DocumentAction::SetTitle(entry.text().to_string())).expect("Could not send title update event");
        }));
    }

    pub fn set_document_title_label_text<S: ToString>(&self, value: S) {
        self.imp().title_entry.set_text(value.to_string().as_str());
    }

    pub fn load_document(&self, document: Option<&Document>) {
        glib::debug!("Loading document on project layout widget");
        // Clear layout
        self.clear();

        if let Some(document) = document {
            // Populate project structure

            // 1. Add chunk categories entries
            document.chunks().iter().for_each(|chunk| {
                self.add_chunk(*chunk);
            });
        }
    }

    fn clear(&self) {
        let layout = self.imp().layout.get();
        let mut child = layout.first_child();
        while child.is_some() {
            let existing_child = child.unwrap();
            child = existing_child.next_sibling();
            layout.remove(&existing_child);
        }
    }

    // fn get_expanders(&self) -> Vec<gtk::Widget> {
    //     let layout = self.imp().layout.get();
    //     let mut children = vec![];
    //     let mut child = layout.first_child();
    //     while child.is_some() {
    //         let existing_child = child.unwrap();
    //         child = existing_child.next_sibling();
    //         children.push(existing_child);
    //     }
    //     children
    // }

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

        let row = factories::create_row_for_chunk(chunk);
        row.connect_activated(glib::clone!(@weak self as this => move |row| {
            this.send_action(DocumentAction::SelectChunk(row.chunk_id()));
        }));
        expander_row.add_row(&row);
    }

    pub fn remove_chunk<S: ToString>(&self, chunk_id: S) {
        glib::g_debug!(
            G_LOG_DOMAIN,
            "Removing chunk with id {} from project layout",
            chunk_id.to_string()
        );
    }

    pub fn searchbar(&self) -> gtk::SearchBar {
        self.imp().searchbar.get()
    }

    pub fn set_search(&self, value: bool) {
        self.searchbar().set_search_mode(value);
    }

    pub fn set_select(&self, _value: bool) {
        // TODO
    }

    fn send_action(&self, action: DocumentAction) {
        let maybe_channel = self.imp().channel.borrow();
        if let Some(channel) = maybe_channel.as_ref() {
            channel.send(action).expect("Could not send action");
        }
    }
}
