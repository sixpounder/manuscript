use crate::{
    libs::consts::RESOURCE_ID_DATA_KEY,
    models::*,
    services::{DocumentAction, ManuscriptSettings},
    widgets::editors::*,
};
use adw::subclass::prelude::*;
use gtk::{
    gio, glib,
    glib::{clone, Sender},
    prelude::*,
};
use std::cell::{Cell, RefCell};

#[allow(unused)]
const G_LOG_DOMAIN: &str = "ManuscriptEditorViewShell";

mod imp {
    use super::*;
    use glib::{ParamSpec, ParamSpecBoolean, ParamSpecString};
    use once_cell::sync::Lazy;

    /// A widget capable of organizing `ManuscriptEditorView` and manage their lifecycle. Also enables
    /// the display of the chunk properties panel.
    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/editor_view_shell.ui")]
    pub struct ManuscriptEditorViewShell {
        #[template_child]
        pub(super) editor_tab_bar: TemplateChild<adw::TabBar>,

        #[template_child]
        pub(super) editor_tab_view: TemplateChild<adw::TabView>,

        pub(super) channel: RefCell<Option<Sender<DocumentAction>>>,

        pub(super) chunk_props_panel_visible: Cell<bool>,
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
                    ParamSpecBoolean::builder("chunk-props-panel-visible")
                        .default_value(false)
                        .readwrite()
                        .build(),
                    ParamSpecBoolean::builder("has-views")
                        .default_value(false)
                        .read_only()
                        .build(),
                    ParamSpecString::builder("visible-view-name")
                        .read_only()
                        .default_value("placeholder_view")
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "chunk-props-panel-visible" => obj.chunk_props_panel_visible().to_value(),
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

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "chunk-props-panel-visible" => {
                    obj.set_chunk_props_panel_visible(value.get::<bool>().unwrap())
                }
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
        glib::Object::new()
    }

    fn sender(&self) -> Option<Sender<DocumentAction>> {
        let channel = self.imp().channel.borrow();
        channel.as_ref().cloned()
    }

    fn setup_widgets(&self) {
        let settings = ManuscriptSettings::default();
        self.set_chunk_props_panel_visible(settings.chunk_props_panel_visible());

        self.imp().editor_tab_view.connect_close_page(
            clone!(@weak self as this => @default-return false, move |_, _| {
                glib::idle_add_local(move || {
                    this.notify("visible-view-name");
                    glib::ControlFlow::Break
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

    fn chunk_props_panel_visible(&self) -> bool {
        self.imp().chunk_props_panel_visible.get()
    }

    /// Sets wheter the chunk properties panel is visible or not
    /// in this shell.
    fn set_chunk_props_panel_visible(&self, value: bool) {
        let settings = ManuscriptSettings::default();
        settings.set_chunk_props_panel_visible(value);
        self.imp().chunk_props_panel_visible.set(value);
        self.notify("chunk-props-panel-visible");

        let pages = self.editor_tab_view().pages();
        let pages = pages.iter::<adw::TabPage>();

        pages.for_each(|page| {
            let page = page.unwrap();
            let editor_view = page.child();
            let editor_view = editor_view
                .downcast_ref::<ManuscriptEditorView>()
                .expect("Not an editor view");
            editor_view.set_side_panel_visible(value);
        });
    }

    /// Creates a `ManuscriptEditorView` for a given `chunk` and initializes it taking
    /// into account the concrete type of the `chunk`
    fn editor_view_widget_for_chunk(&self, chunk: &dyn DocumentChunk) -> ManuscriptEditorView {
        let child_widget: *mut dyn crate::widgets::editors::prelude::EditorWidgetProtocol =
            match chunk.chunk_type() {
                ChunkType::Manifest => {
                    let manifest = chunk
                        .as_any()
                        .downcast_ref::<DocumentManifest>()
                        .expect("Expected manifest, got non compliant type");
                    let editor = ManuscriptProjectSettingsEditor::new(manifest, self.sender());
                    editor.set_halign(gtk::Align::Fill);
                    editor.set_valign(gtk::Align::Fill);
                    editor.set_hexpand(true);
                    // editor.upcast::<gtk::Widget>()
                    Box::into_raw(Box::new(editor))
                }
                ChunkType::Chapter => {
                    let text_view = ManuscriptTextEditor::new(chunk, self.sender());
                    text_view.set_halign(gtk::Align::Fill);
                    text_view.set_valign(gtk::Align::Fill);
                    text_view.set_hexpand(true);
                    if let Some(chapter) = chunk.as_any().downcast_ref::<Chapter>() {
                        text_view.init(chunk.id().into(), Some(chapter.buffer().clone()));
                    } else {
                        text_view.init(chunk.id().into(), None);
                    }
                    // text_view.upcast::<gtk::Widget>()
                    Box::into_raw(Box::new(text_view))
                }
                ChunkType::CharacterSheet => {
                    let editor = ManuscriptCharacterSheetEditor::new(chunk, self.sender());
                    editor.set_halign(gtk::Align::Fill);
                    editor.set_valign(gtk::Align::Fill);
                    editor.set_hexpand(true);
                    // editor.set_width_request(250);
                    // editor.upcast::<gtk::Widget>()
                    Box::into_raw(Box::new(editor))
                }
            };

        let child_widget = unsafe { Box::from_raw(child_widget) };
        let settings = ManuscriptSettings::default();
        let view = ManuscriptEditorView::new(child_widget);
        view.set_side_panel_visible(settings.chunk_props_panel_visible());
        view
    }

    fn page_for_chunk(&self, chunk: &(impl DocumentChunk + ?Sized)) -> Option<adw::TabPage> {
        self.page_by_resource_id(chunk.id().to_string())
    }

    fn page_by_resource_id(&self, id: String) -> Option<adw::TabPage> {
        let editor_view = self.editor_tab_view();
        let page_list_iterator = editor_view.pages();
        let mut page_list_iterator = page_list_iterator.iter::<adw::TabPage>();
        page_list_iterator
            .find(|page| {
                if let Ok(page) = page {
                    let maybe_data = unsafe { page.data::<String>(RESOURCE_ID_DATA_KEY) };
                    if let Some(inner_data) = maybe_data {
                        let inner_data = unsafe { inner_data.as_ref() };
                        if *inner_data == id {
                            return true;
                        }
                    }
                }

                false
            })
            .map(|page| page.unwrap())
    }

    /// Adds a `TabPage` to the shell for `chunk`, without selecting it
    pub fn add_chunk_page(&self, chunk: &dyn DocumentChunk) -> adw::TabPage {
        let page = self
            .page_by_resource_id(chunk.id().to_string())
            .unwrap_or_else(|| {
                let view_child = self.editor_view_widget_for_chunk(chunk);
                let view = self.editor_tab_view();
                let page = view.append(&view_child);
                page.set_title(chunk.heading().as_str());
                unsafe { page.set_data(RESOURCE_ID_DATA_KEY, chunk.id().to_string()) };
                page
            });

        self.notify("visible-view-name");
        page
    }

    /// Adds a `TabPage` to the shell for `chunk` and selects it
    pub fn add_and_select_chunk_page(&self, chunk: &dyn DocumentChunk) {
        let view = self.editor_tab_view();
        let selected_page = &self.add_chunk_page(chunk);
        view.set_selected_page(selected_page);
        selected_page.child().grab_focus();
    }

    /// Selects a `TabPage` for a given `chunk`. If that page does not exists, creates it and
    /// then selects it
    pub fn select_chunk_page(&self, chunk: &dyn DocumentChunk) {
        if let Some(page) = self.page_for_chunk(chunk) {
            self.editor_tab_view().set_selected_page(&page);
        } else {
            self.add_and_select_chunk_page(chunk);
        }
    }

    /// Selects a `TabPage` for a given `chunk`.
    pub fn select_page(&self, page: &adw::TabPage) {
        self.editor_tab_view().set_selected_page(page);
    }

    /// Closes the `TabPage` corresponding to a given `chunk`.
    pub fn close_page(&self, chunk: &dyn DocumentChunk) {
        if let Some(page) = self.page_for_chunk(chunk) {
            self.editor_tab_view().close_page(&page);
        }
    }

    /// Closes the `TabPage` with the given resource `id`, if any exists.
    pub fn close_page_by_id(&self, id: String) {
        if let Some(page) = self.page_by_resource_id(id) {
            self.editor_tab_view().close_page(&page);
        }
    }

    /// Closes all currently opened pages
    pub fn clear(&self) {
        let editor_view = self.editor_tab_view();
        let page_list_iterator = editor_view.pages();
        let page_list_iterator = page_list_iterator.iter::<adw::TabPage>();
        let pages = page_list_iterator
            .map(|p| p.expect("Failed to retrieve TabPage for closing"))
            .collect::<Vec<adw::TabPage>>();
        pages.iter().for_each(|p| {
            editor_view.close_page(p);
        });
    }

    /// Updates a page with the state of a given `chunk`, if that page
    /// exists
    pub fn update_page(&self, chunk: &(impl DocumentChunk + ?Sized)) {
        if let Some(page) = self.page_for_chunk(chunk) {
            page.set_title(chunk.heading().as_str());
        }
    }

    pub fn set_channel(&self, sender: Sender<DocumentAction>) {
        self.imp().channel.replace(Some(sender));
    }
}
