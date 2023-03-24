use crate::{
    libs::files::{with_file_open_dialog, with_file_save_dialog},
    models::*,
    services::{DocumentManager, ManuscriptSettings},
    widgets::{
        dialogs::ManuscriptDestroyConfirmDialog, ManuscriptEditorViewShell,
        ManuscriptProjectLayout, ManuscriptThemeSwitcher, ManuscriptWelcomeView,
    },
};
use adw::{prelude::*, subclass::prelude::*};
use gtk::{gio, glib::closure_local};
use std::{cell::Cell, ops::Deref};

const G_LOG_DOMAIN: &str = "ManuscriptWindow";
const PROJECT_VIEW_NAME: &str = "project-view";
const WELCOME_VIEW_NAME: &str = "welcome-view";

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean};
    use once_cell::sync::Lazy;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/window.ui")]
    pub struct ManuscriptWindow {
        #[template_child]
        pub(super) toast_overlay: TemplateChild<adw::ToastOverlay>,

        #[template_child]
        pub(super) primary_menu_button: TemplateChild<gtk::MenuButton>,

        #[template_child]
        pub(super) primary_menu_button_alt: TemplateChild<gtk::MenuButton>,

        #[template_child]
        pub(super) main_stack: TemplateChild<gtk::Stack>,

        #[template_child]
        pub(super) welcome_view: TemplateChild<ManuscriptWelcomeView>,

        #[template_child]
        pub(super) project_layout: TemplateChild<ManuscriptProjectLayout>,

        #[template_child]
        pub(super) command_palette_overlay: TemplateChild<gtk::Overlay>,

        #[template_child]
        pub(super) editor_view: TemplateChild<ManuscriptEditorViewShell>,

        #[template_child]
        pub(super) flap: TemplateChild<adw::Flap>,

        pub(super) theme_switcher: ManuscriptThemeSwitcher,

        pub(super) settings: ManuscriptSettings,

        pub(super) document_manager: DocumentManager,

        pub(super) search_mode: Cell<bool>,

        pub(super) select_mode: Cell<bool>,

        pub(super) close_anyway: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptWindow {
        const NAME: &'static str = "ManuscriptWindow";
        type Type = super::ManuscriptWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                toast_overlay: TemplateChild::default(),
                primary_menu_button: TemplateChild::default(),
                primary_menu_button_alt: TemplateChild::default(),
                main_stack: TemplateChild::default(),
                welcome_view: TemplateChild::default(),
                project_layout: TemplateChild::default(),
                command_palette_overlay: TemplateChild::default(),
                editor_view: TemplateChild::default(),
                flap: TemplateChild::default(),
                theme_switcher: ManuscriptThemeSwitcher::new(),
                settings: ManuscriptSettings::default(),
                document_manager: DocumentManager::default(),
                search_mode: Cell::default(),
                select_mode: Cell::default(),
                close_anyway: Cell::new(false),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.bind_template_instance_callbacks();

            klass.install_action("app.new-project", None, move |win, _, _| {
                win.new_project();
            });

            klass.install_action("app.open-project", None, move |win, _, _| {
                win.open_project(false);
            });

            klass.install_action("project.save", None, move |win, _, _| {
                win.save_project();
            });

            klass.install_action("project.save-as", None, move |win, _, _| {
                win.save_project_as();
            });

            klass.install_action("project.close", None, move |win, _, _| {
                win.close_project(false);
            });

            klass.install_action("win.toggle-command-palette", None, move |win, _, _| {
                win.toggle_command_palette();
            });

            klass.install_action("project.add-chapter", None, move |win, _, _| {
                win.add_chapter();
            });

            klass.install_action("project.add-character-sheet", None, move |win, _, _| {
                win.add_character_sheet();
            });

            klass.install_property_action("project.search", "project-search");
            klass.install_property_action("project.select", "project-select");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_widgets();
            obj.restore_window_state();
            obj.connect_events();
        }

        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new("project-search", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("project-select", "", "", false, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "project-search" => obj.search_mode().to_value(),
                "project-select" => obj.select().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "project-search" => obj.set_search_mode(value.get::<bool>().unwrap()),
                "project-select" => obj.set_select(value.get::<bool>().unwrap()),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for ManuscriptWindow {}
    impl WindowImpl for ManuscriptWindow {}
    impl ApplicationWindowImpl for ManuscriptWindow {}
    impl AdwApplicationWindowImpl for ManuscriptWindow {}
}

glib::wrapper! {
    pub struct ManuscriptWindow(ObjectSubclass<imp::ManuscriptWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

// Public APIs
impl ManuscriptWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new(&[("application", application)])
    }

    pub fn document_manager(&self) -> &DocumentManager {
        &self.imp().document_manager
    }

    pub fn editor_view(&self) -> ManuscriptEditorViewShell {
        self.imp().editor_view.get()
    }

    pub fn project_layout(&self) -> ManuscriptProjectLayout {
        self.imp().project_layout.get()
    }
}

// Private APIs
impl ManuscriptWindow {
    fn restore_window_state(&self) {
        let settings = &self.imp().settings;
        self.set_default_size(settings.window_width(), settings.window_height());
    }

    fn connect_events(&self) {
        let dm = self.document_manager();

        dm.connect_closure(
            "document-loaded",
            false,
            closure_local!(@strong self as this => move |_: DocumentManager| {
                this.on_document_loaded();
            }),
        );

        dm.connect_closure(
            "document-unloaded",
            false,
            closure_local!(@strong self as this => move |_: DocumentManager| {
                this.on_document_unloaded();
            }),
        );

        dm.connect_closure(
            "chunk-added",
            false,
            closure_local!(@strong self as this => move |_obj: DocumentManager, id: String| {
                this.on_chunk_added(id);
            }),
        );

        dm.connect_closure(
            "chunk-removed",
            false,
            closure_local!(@strong self as this => move |_obj: DocumentManager, id: String| {
                this.on_chunk_removed(id);
            }),
        );

        dm.connect_closure(
            "chunk-selected",
            false,
            closure_local!(@strong self as this => move |_obj: DocumentManager, id: String| {
                this.on_chunk_selected(id);
            }),
        );

        dm.connect_closure(
            "chunk-updated",
            false,
            closure_local!(@strong self as this => move |_obj: DocumentManager, id: String| {
                this.on_chunk_updated(id);
            }),
        );

        dm.connect_closure(
            "chunk-stats-updated",
            false,
            closure_local!(@strong self as this => move |_dm: DocumentManager, id: String, words_count: u64, reading_minutes: u64, reading_seconds: u64| {
                this.on_chunk_stats_updated(id, words_count, (reading_minutes, reading_seconds));
            })
        );

        self.imp().project_layout.searchbar().connect_notify_local(
            Some("search-mode-enabled"),
            glib::clone!(@weak self as win => move |searchbar, _| {
                win.set_search_mode(searchbar.is_search_mode());
            }),
        );

        self.connect_close_request(move |window| {
            glib::g_debug!(G_LOG_DOMAIN, "Saving window state");
            let width = window.default_size().0;
            let height = window.default_size().1;
            let settings = ManuscriptSettings::default();
            settings.set_window_width(width);
            settings.set_window_height(height);
            glib::signal::Inhibit(false)
        });
    }

    fn setup_widgets(&self) {
        if let Some(popover_menu) = self
            .imp()
            .primary_menu_button
            .popover()
            .and_downcast_ref::<gtk::PopoverMenu>()
        {
            popover_menu.add_child(&ManuscriptThemeSwitcher::new(), "themeswitcher");
        }

        if let Some(popover_menu) = self
            .imp()
            .primary_menu_button_alt
            .popover()
            .and_downcast_ref::<gtk::PopoverMenu>()
        {
            popover_menu.add_child(&self.imp().theme_switcher, "themeswitcher");
        }

        let project_layout = self.imp().project_layout.get();
        project_layout.set_channel(self.document_manager().action_sender());

        let editor_view = self.editor_view();
        editor_view.set_channel(self.document_manager().action_sender());

        self.update_actions();
    }

    fn add_toast(&self, msg: String) {
        let toast = adw::Toast::new(&msg);
        self.imp().toast_overlay.add_toast(&toast);
    }

    fn new_project(&self) {
        self.document_manager()
            .load_document(None)
            .expect("Could not load empty document");
        self.imp()
            .main_stack
            .set_visible_child_name(PROJECT_VIEW_NAME);
    }

    fn open_project(&self, pass: bool) {
        let dm = self.document_manager();
        if !(dm.is_sync() || pass) {
            let dialog = ManuscriptDestroyConfirmDialog::new(self.upcast_ref::<gtk::Window>());
            dialog.connect_response(
                None,
                glib::clone!(@strong self as this => move |_dialog, res| {
                    if res == "save" {
                        this.save_project();
                        this.open_project(true);
                    } else if res == "discard" {
                        this.open_project(true);
                    }
                }),
            );
            dialog.show();
        } else {
            with_file_open_dialog(glib::clone!(@strong self as win => move |path| {
                let imp = win.imp();
                let dm = win.document_manager();

                if dm.has_document() && dm.unload_document().is_ok() {
                    win.editor_view().clear();
                    win.project_layout().clear();
                }
                match dm.load_document(Some(path)) {
                    Ok(_) => {
                        imp.main_stack.set_visible_child_name(PROJECT_VIEW_NAME);

                        // Update last opened document
                        let settings = &imp.settings;
                        if dm.has_backend() {
                            let backend_path = dm.backend_file();
                            let backend_path = backend_path.as_ref().unwrap();
                            let backend_path = backend_path.path().unwrap();
                            let backend_path = backend_path.as_path().to_str().unwrap();
                            let backend_path = backend_path.to_string();

                            settings.set_last_opened_document(&backend_path);
                            glib::g_debug!(G_LOG_DOMAIN, "Updated last opened document with {backend_path}");
                        }
                    },
                    Err(error) => {
                        match error {
                            ManuscriptError::Open(path) => {
                                win.add_toast(format!("Unreadable file: {}", path));
                            },
                            _ => ()
                        }
                    }
                }
            }));
        }
    }

    fn save_project(&self) {
        let dm = self.document_manager();
        if dm.has_document() {
            if dm.has_backend() {
                match dm.sync() {
                    Ok(bytes_written) => {
                        glib::g_info!(
                            G_LOG_DOMAIN,
                            "Project saved - {bytes_written} bytes written"
                        );
                    }
                    Err(error) => {
                        glib::g_warning!(G_LOG_DOMAIN, "Problem when saving project - {:?}", error);
                    }
                }
            } else {
                with_file_save_dialog(glib::clone!(@strong self as win => move |path| {
                    win.document_manager().set_backend_path(path);
                    if let Err(_error) = win.document_manager().sync() {
                        win.add_toast("Could not save file".into());
                    }
                }));
            }
        } else {
            glib::g_warning!(G_LOG_DOMAIN, "Could not acquire document for saving");
        }
    }

    fn save_project_as(&self) {
        let dm = self.document_manager();
        if dm.has_document() {
            with_file_save_dialog(glib::clone!(@strong self as win => move |path| {
                let settings = &win.imp().settings;
                settings.set_last_opened_document(&path);
                win.document_manager().set_backend_path(path);
                if let Err(_error) = win.document_manager().sync() {
                    win.add_toast("Could not save file".into());
                }
            }));
        }
    }

    fn close_project(&self, pass: bool) {
        let dm = self.document_manager();
        if dm.has_document() {
            if !(dm.is_sync() || pass) {
                let dialog = ManuscriptDestroyConfirmDialog::new(self.upcast_ref::<gtk::Window>());
                dialog.connect_response(
                    None,
                    glib::clone!(@strong self as this => move |_dialog, res| {
                        if res == "save" {
                            this.save_project();
                            this.close_project(true);
                        } else if res == "discard" {
                            this.close_project(true);
                        }
                    }),
                );
                dialog.show();
            } else if dm.unload_document().is_ok() {
                self.editor_view().clear();
                self.project_layout().clear();
                self.imp()
                    .main_stack
                    .set_visible_child_name(WELCOME_VIEW_NAME);
            }
        }
    }

    fn on_document_loaded(&self) {
        self.update_actions();
        glib::idle_add_local(
            glib::clone!(@weak self as this => @default-return glib::Continue(false), move || {
                let imp = this.imp();
                let new_document = imp
                    .document_manager
                    .document_ref()
                    .expect("Could not lock document");
                let new_document = new_document.deref();

                // Update project layout
                imp.project_layout.load_document(new_document.as_ref());
                glib::Continue(false)
            }),
        );
    }

    fn on_document_unloaded(&self) {
        self.update_actions();
    }

    fn on_chunk_added(&self, id: String) {
        if let Ok(lock) = self.document_manager().document_ref() {
            if let Some(document) = lock.as_ref() {
                let imp = self.imp();
                let added_chunk = document.get_chunk_ref(id.as_str()).unwrap();
                imp.editor_view.add_and_select_page(added_chunk);
                imp.project_layout.add_chunk(added_chunk);

                // Ensure flap closes and editor view gets revealed if in folded mode
                let flap = self.flap();
                if flap.is_folded() {
                    flap.set_reveal_flap(false);
                }
            }
        }
    }

    fn on_chunk_removed(&self, id: String) {
        let imp = self.imp();
        imp.project_layout.remove_chunk(id.clone());
        imp.editor_view.close_page_by_id(id);
    }

    fn on_chunk_selected(&self, id: String) {
        if let Ok(lock) = self.document_manager().document_ref() {
            if let Some(document) = &*lock {
                let selected_chunk = document.get_chunk_ref(id.as_str()).unwrap();
                self.editor_view().select_page(selected_chunk);

                // Ensure flap closes and editor view gets revealed if in folded mode
                let flap = self.flap();
                if flap.is_folded() {
                    flap.set_reveal_flap(false);
                }
            }
        }
    }

    fn on_chunk_updated(&self, id: String) {
        self.update_layout_chunk_row(id.clone());
        self.update_editor_view_shell(id);
        // TODO: maybe tick for autosave here?
    }

    fn on_chunk_stats_updated(&self, id: String, words_count: u64, reading_time: (u64, u64)) {
        self.update_layout_chunk_row_reading_stats(id, words_count, reading_time);
    }

    fn search_mode(&self) -> bool {
        self.imp().search_mode.get()
    }

    fn set_search_mode(&self, value: bool) {
        let imp = self.imp();
        if value != imp.search_mode.replace(value) {
            imp.project_layout.set_search(value);
            self.notify("project-search");
        }
    }

    fn select(&self) -> bool {
        self.imp().select_mode.get()
    }

    fn set_select(&self, value: bool) {
        let imp = self.imp();
        if value != imp.select_mode.replace(value) {
            imp.project_layout.set_select(value);
            self.notify("project-select");
        }
    }

    fn add_chapter(&self) {
        glib::g_debug!(G_LOG_DOMAIN, "Adding empty chapter to the project");
        self.document_manager().add_chunk(Chapter::default());
    }

    fn add_character_sheet(&self) {
        glib::g_debug!(G_LOG_DOMAIN, "Adding empty character sheet to the project");
        self.document_manager().add_chunk(CharacterSheet::default());
    }

    fn update_layout_chunk_row(&self, id: String) {
        if let Ok(lock) = self.document_manager().document_ref() {
            if let Some(document) = &*lock {
                if let Some(chunk) = document.get_chunk_ref(&id) {
                    if let Some(row) = self.project_layout().chunk_row(chunk) {
                        let the_chunk = Some(chunk);
                        row.update_chunk(the_chunk);
                    } else {
                        glib::g_warning!(
                            G_LOG_DOMAIN,
                            "Could not find chunk row to update for id {id}, skipping"
                        );
                    }
                } else {
                    glib::g_warning!(
                        G_LOG_DOMAIN,
                        "Chunk {id} was notified to be updated, but then was not found in document"
                    );
                }
            } else {
                glib::g_warning!(G_LOG_DOMAIN, "Could not lock document");
            }
        } else {
            glib::g_warning!(
                G_LOG_DOMAIN,
                "Chunk {id} was notified to be updated, but then was not found in document"
            );
        }
    }

    fn update_layout_chunk_row_reading_stats(
        &self,
        id: String,
        words_count: u64,
        _reading_time: (u64, u64),
    ) {
        if let Ok(lock) = self.document_manager().document_ref() {
            if let Some(document) = &*lock {
                if let Some(chunk) = document.get_chunk_ref(&id) {
                    if let Some(row) = self.project_layout().chunk_row(chunk) {
                        row.update_chunk_reading_stats(chunk, words_count);
                    } else {
                        glib::g_warning!(
                            G_LOG_DOMAIN,
                            "Could not find chunk row to update for id {id}, skipping"
                        );
                    }
                } else {
                    glib::g_warning!(
                        G_LOG_DOMAIN,
                        "Chunk {id} was notified to be updated, but then was not found in document"
                    );
                }
            } else {
                glib::g_warning!(G_LOG_DOMAIN, "Could not lock document");
            }
        } else {
            glib::g_warning!(
                G_LOG_DOMAIN,
                "Chunk {id} was notified to be updated, but then was not found in document"
            );
        }
    }

    fn update_editor_view_shell(&self, id: String) {
        if let Ok(lock) = self.document_manager().document_ref() {
            if let Some(document) = &*lock {
                if let Some(chunk) = document.get_chunk_ref(&id) {
                    self.editor_view().update_page(chunk);
                } else {
                    glib::g_warning!(
                        G_LOG_DOMAIN,
                        "Chunk {id} was notified to be updated, but then was not found in document"
                    );
                }
            } else {
                glib::g_warning!(G_LOG_DOMAIN, "Could not lock document");
            }
        } else {
            glib::g_warning!(
                G_LOG_DOMAIN,
                "Chunk {id} was notified to be updated, but then was not found in document"
            );
        }
    }

    fn toggle_command_palette(&self) {
        const COMMAND_PALETTE_CLASS: &str = "command-palette";
        let main_stack_style_context = self.imp().main_stack.style_context();
        if main_stack_style_context.has_class(COMMAND_PALETTE_CLASS) {
            main_stack_style_context.remove_class(COMMAND_PALETTE_CLASS);
        } else {
            main_stack_style_context.add_class(COMMAND_PALETTE_CLASS);
        }
    }

    fn update_actions(&self) {
        let dm = self.document_manager();
        self.action_set_enabled("project.compile", dm.has_document());
        self.action_set_enabled("project.save", dm.has_document());
        self.action_set_enabled("project.save-as", dm.has_document());
        self.action_set_enabled("project.close", dm.has_document());
    }

    pub fn flap(&self) -> adw::Flap {
        self.imp().flap.get()
    }

    pub fn can_close(&self) -> bool {
        self.imp().close_anyway.get()
            || (!self.document_manager().has_document() || self.document_manager().is_sync())
    }
}

#[gtk::template_callbacks]
impl ManuscriptWindow {
    #[template_callback]
    fn on_remove_selected_activated(&self, ids: Vec<String>) {
        let dm = self.document_manager();
        ids.iter().for_each(|id| {
            dm.remove_chunk(id);
        });
    }

    #[template_callback]
    fn on_close_request(&self) -> bool {
        if self.can_close() {
            false
        } else {
            let dialog = ManuscriptDestroyConfirmDialog::new(self.upcast_ref::<gtk::Window>());
            dialog.connect_response(
                None,
                glib::clone!(@strong self as this => move |_dialog, res| {
                    if res == "save" {
                        this.save_project();
                        this.imp().close_anyway.set(true);
                        this.close();
                    } else if res == "discard" {
                        this.imp().close_anyway.set(true);
                        this.close();
                    }
                }),
            );
            dialog.show();
            true
        }
    }
}
