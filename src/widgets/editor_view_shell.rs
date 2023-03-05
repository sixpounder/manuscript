use crate::{consts::CHUNK_ID_DATA_KEY, models::*, services::DocumentAction, widgets::editors::*};
use adw::subclass::prelude::*;
use gtk::{
    gio, glib,
    glib::{clone, Sender},
    prelude::*,
};
use std::cell::RefCell;

const G_LOG_DOMAIN: &str = "ManuscriptEditorViewShell";

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/editor_view_shell.ui")]
    pub struct ManuscriptEditorViewShell {
        #[template_child]
        pub(super) editor_tab_bar: TemplateChild<adw::TabBar>,

        #[template_child]
        pub(super) editor_tab_view: TemplateChild<adw::TabView>,

        pub(super) channel: RefCell<Option<Sender<DocumentAction>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptEditorViewShell {
        const NAME: &'static str = "ManuscriptEditorViewShell";
        type Type = super::ManuscriptEditorViewShell;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptEditorViewShell {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_widgets();
        }

        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new("has-views", "", "", false, ParamFlags::READABLE),
                    ParamSpecString::new(
                        "visible-view-name",
                        "",
                        "",
                        "placeholder_view".into(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "has-views" => obj.editor_tab_view().n_pages().is_positive().to_value(),
                "visible-view-name" => {
                    if obj.editor_tab_view().n_pages().is_positive() {
                        "editors_view".into()
                    } else {
                        "placeholder_view".into()
                    }
                }
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, _value: &glib::Value, pspec: &ParamSpec) {
            let _obj = self.obj();
            match pspec.name() {
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ManuscriptEditorViewShell {}
}

glib::wrapper! {
    pub struct ManuscriptEditorViewShell(ObjectSubclass<imp::ManuscriptEditorViewShell>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ManuscriptEditorViewShell {
    fn default() -> Self {
        Self::new()
    }
}

impl ManuscriptEditorViewShell {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    fn setup_widgets(&self) {
        self.imp().editor_tab_view.connect_close_page(
            clone!(@weak self as this => @default-return false, move |_, _| {
                glib::idle_add_local(move || {
                    this.notify("visible-view-name");
                    glib::Continue(false)
                });
                false
            }),
        );
    }

    fn tab_bar(&self) -> adw::TabBar {
        self.imp().editor_tab_bar.get()
    }

    fn editor_tab_view(&self) -> adw::TabView {
        self.tab_bar().view().expect("Could not get tab view")
    }

    fn editor_view_widget_for_chunk(&self, chunk: &dyn DocumentChunk) -> gtk::Widget {
        match chunk.chunk_type() {
            ChunkType::Manifest => todo!(),
            ChunkType::Chapter => {
                let text_view = ManuscriptTextEditor::new(self.sender());
                text_view.set_halign(gtk::Align::Fill);
                text_view.set_valign(gtk::Align::Fill);
                text_view.set_hexpand(true);
                text_view.init(chunk.id().into(), None);
                text_view.upcast::<gtk::Widget>()
            }
            ChunkType::CharacterSheet => {
                let editor = ManuscriptCharacterSheetEditor::new();
                editor.set_halign(gtk::Align::Fill);
                editor.set_valign(gtk::Align::Fill);
                editor.set_hexpand(true);
                editor.upcast::<gtk::Widget>()
            }
        }
    }

    fn page_for_chunk(&self, chunk: &dyn DocumentChunk) -> Option<adw::TabPage> {
        let editor_view = self.editor_tab_view();
        let page_list_iterator = editor_view.pages();
        let mut page_list_iterator = page_list_iterator
            .iter::<adw::TabPage>()
            .expect("No iterator for view pages");
        page_list_iterator
            .find(|page| {
                if let Ok(page) = page {
                    let maybe_data = unsafe { page.data::<String>(CHUNK_ID_DATA_KEY) };
                    if let Some(inner_data) = maybe_data {
                        let inner_data = unsafe { inner_data.as_ref() };
                        if *inner_data == chunk.id() {
                            return true;
                        }
                    }
                }

                false
            })
            .map(|page| page.unwrap())
    }

    pub fn add_page(&self, chunk: &dyn DocumentChunk) -> adw::TabPage {
        let view_child = self.editor_view_widget_for_chunk(chunk);
        let view = self.editor_tab_view();
        let page = view.append(&view_child);

        page.set_title(chunk.safe_title());
        unsafe { page.set_data(CHUNK_ID_DATA_KEY, chunk.id().to_string()) };

        self.notify("visible-view-name");

        page
    }

    pub fn add_and_select_page(&self, chunk: &dyn DocumentChunk) {
        let view = self.editor_tab_view();
        view.set_selected_page(&self.add_page(chunk));
    }

    pub fn select_page(&self, chunk: &dyn DocumentChunk) {
        if let Some(page) = self.page_for_chunk(chunk) {
            self.editor_tab_view().set_selected_page(&page);
        } else {
            glib::g_info!(G_LOG_DOMAIN, "No view found for {}", chunk.id());
            // TODO: page should be created
        }
    }

    pub fn set_channel(&self, sender: Sender<DocumentAction>) {
        self.imp().channel.replace(Some(sender));
    }

    fn sender(&self) -> Option<Sender<DocumentAction>> {
        let channel = self.imp().channel.borrow();
        if let Some(sender) = channel.as_ref() {
            Some(sender.clone())
        } else {
            None
        }
    }
}
