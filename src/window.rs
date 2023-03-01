/* window.rs
 *
 * Copyright 2023 Andrea
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::{
    config::{APPLICATION_G_PATH, G_LOG_DOMAIN},
    libs::files::with_file_open_dialog,
    models::*,
    services::{i18n::i18n, DocumentManager, ManuscriptSettings},
    widgets::{ManuscriptProjectLayout, ManuscriptTextEditor, ManuscriptCharacterSheetEditor, ManuscriptWelcomeView},
};
use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gdk, gio, glib::closure_local};
use std::{cell::Cell, ops::Deref, rc::Rc};

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/window.ui")]
    pub struct ManuscriptWindow {
        #[template_child]
        pub(super) toast_overlay: TemplateChild<adw::ToastOverlay>,

        #[template_child]
        pub(super) main_stack: TemplateChild<gtk::Stack>,

        #[template_child]
        pub(super) welcome_view: TemplateChild<ManuscriptWelcomeView>,

        #[template_child]
        pub(super) editor_tab_bar: TemplateChild<adw::TabBar>,

        #[template_child]
        pub(super) project_layout: TemplateChild<ManuscriptProjectLayout>,

        #[template_child]
        pub(super) command_palette_overlay: TemplateChild<gtk::Overlay>,

        pub(super) style_manager: adw::StyleManager,

        pub(super) provider: gtk::CssProvider,

        pub(super) settings: ManuscriptSettings,

        pub(super) document_manager: Rc<DocumentManager>,

        pub(super) search_mode: Cell<bool>,

        pub(super) select_mode: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptWindow {
        const NAME: &'static str = "ManuscriptWindow";
        type Type = super::ManuscriptWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                toast_overlay: TemplateChild::default(),
                main_stack: TemplateChild::default(),
                welcome_view: TemplateChild::default(),
                editor_tab_bar: TemplateChild::default(),
                project_layout: TemplateChild::default(),
                command_palette_overlay: TemplateChild::default(),
                style_manager: adw::StyleManager::default(),
                provider: gtk::CssProvider::default(),
                settings: ManuscriptSettings::default(),
                document_manager: Rc::new(DocumentManager::default()),
                search_mode: Cell::default(),
                select_mode: Cell::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("win.new-project", None, move |win, _, _| {
                win.new_project();
            });

            klass.install_action("win.open-project", None, move |win, _, _| {
                win.open_project();
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
            obj.setup_provider();
            obj.setup_widgets();
            obj.restore_window_state();
            obj.connect_events();
            obj.update_widgets();
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

impl ManuscriptWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new(&[("application", application)])
    }

    pub fn document_manager(&self) -> Rc<DocumentManager> {
        self.imp().document_manager.clone()
    }

    fn add_toast(&self, msg: String) {
        let toast = adw::Toast::new(&msg);
        self.imp().toast_overlay.add_toast(&toast);
    }

    fn new_project(&self) {
        self.imp().main_stack.set_visible_child_name("project-view");
    }

    fn open_project(&self) {
        with_file_open_dialog(
            glib::clone!(@strong self as win => move |document| {
                win.imp().document_manager.set_document(document).expect("Could not set document");
            }),
            glib::clone!(@strong self as win => move |err| {
                glib::g_critical!(G_LOG_DOMAIN, "{}", err);
                win.add_toast(err);
            }),
        );
    }

    fn restore_window_state(&self) {
        let settings = &self.imp().settings;
        self.set_default_size(settings.window_width(), settings.window_height());
    }

    fn update_widgets(&self) {
        let win = self.imp().instance();
        if self.imp().style_manager.is_dark() {
            win.style_context().add_class("dark");
        } else {
            win.style_context().remove_class("dark");
        }
    }

    fn connect_events(&self) {
        let dm = self.imp().document_manager.as_ref();

        dm.connect_closure(
            "document-loaded",
            false,
            closure_local!(@strong self as this => move |_: DocumentManager| {
                this.on_document_loaded();
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

        self.imp().style_manager.connect_dark_notify(
            glib::clone!(@strong self as this => move |_sm| {
                this.update_widgets();
            }),
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

    fn setup_provider(&self) {
        let imp = self.imp();
        imp.provider
            .load_from_resource(format!("{}/{}", APPLICATION_G_PATH, "style.css").as_str());
        if let Some(display) = gdk::Display::default() {
            gtk::StyleContext::add_provider_for_display(&display, &imp.provider, 400);
        }
    }

    fn setup_widgets(&self) {
        let project_layout = self.imp().project_layout.get();
        project_layout.set_channel(self.document_manager().action_sender());
    }

    fn editor_view_widget_for_chunk(&self, chunk: &dyn DocumentChunk) -> gtk::Widget {
        match chunk.chunk_type() {
            ChunkType::Chapter => {
                let text_view = ManuscriptTextEditor::new();
                text_view.set_halign(gtk::Align::Fill);
                text_view.set_valign(gtk::Align::Fill);
                text_view.set_hexpand(true);
                text_view.init(chunk.id().into(), None);
                text_view.upcast::<gtk::Widget>()
            },
            ChunkType::CharacterSheet => ManuscriptCharacterSheetEditor::new().upcast::<gtk::Widget>()
        }
    }

    fn on_document_loaded(&self) {
        // Get a reference to the new document
        glib::idle_add_local(
            glib::clone!(@weak self as this => @default-return glib::Continue(false), move || {
                let new_document = this
                    .imp()
                    .document_manager
                    .document_ref()
                    .expect("Could not lock document");
                let new_document = new_document.deref();

                // Update project layout
                this.imp().project_layout.load_document(new_document);
                glib::Continue(false)
            }),
        );
    }

    fn on_chunk_added(&self, id: String) {
        if let Ok(lock) = self.imp().document_manager.document_ref() {
            let added_chunk = lock.get_chunk_ref(id.as_str()).unwrap();

            let view_child = self.editor_view_widget_for_chunk(added_chunk);

            let page_title = added_chunk.title().unwrap_or(added_chunk.default_title());
            glib::g_debug!(G_LOG_DOMAIN, "on_chunk_added - Page title -> {page_title}");
            let view = self.editor_view();
            let page = view.append(&view_child);
            page.set_title(page_title);
            view.set_selected_page(&page);

            self.imp().project_layout.add_chunk(added_chunk);
        }
    }

    fn on_chunk_removed(&self, id: String) {
        if let Ok(lock) = self.imp().document_manager.document_ref() {
            let removed_chunk = lock.get_chunk_ref(id.as_str()).unwrap();
            self.imp().project_layout.remove_chunk(removed_chunk.id());
        }
    }

    fn tab_bar(&self) -> adw::TabBar {
        self.imp().editor_tab_bar.get()
    }

    fn editor_view(&self) -> adw::TabView {
        self.tab_bar().view().expect("Could not get tab view")
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

    fn toggle_command_palette(&self) {
        const COMMAND_PALETTE_CLASS: &str = "command-palette";
        let main_stack_style_context = self.imp().main_stack.style_context();
        if main_stack_style_context.has_class(COMMAND_PALETTE_CLASS) {
            main_stack_style_context.remove_class(COMMAND_PALETTE_CLASS);
        } else {
            main_stack_style_context.add_class(COMMAND_PALETTE_CLASS);
        }
    }

    fn set_select(&self, value: bool) {
        let imp = self.imp();
        if value != imp.select_mode.replace(value) {
            imp.project_layout.set_select(value);
            self.notify("project-select");
        }
    }

    fn set_document(&self, document: Document) {
        self.imp()
            .document_manager
            .set_document(document)
            .expect("Could not set document");
    }

    fn add_chapter(&self) {
        glib::g_debug!(G_LOG_DOMAIN, "Adding empty chapter sheet to the project");
        self.imp().document_manager.add_chunk(Chapter::default());
    }

    fn add_character_sheet(&self) {
        glib::g_debug!(G_LOG_DOMAIN, "Adding empty character sheet to the project");
        self.imp()
            .document_manager
            .add_chunk(CharacterSheet::default());
    }
}
