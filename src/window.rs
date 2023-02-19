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
    config::G_LOG_DOMAIN,
    lib::files::with_file_open_dialog,
    models::*,
    services::{i18n::i18n, DocumentManager, ManuscriptSettings},
    widgets::{ManuscriptProjectLayout, ManuscriptTextEditor, ManuscriptWelcomeView},
};
use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib::closure_local};
use std::{io::Read, ops::Deref, rc::Rc};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/window.ui")]
    pub struct ManuscriptWindow {
        #[template_child]
        pub(super) toast_overlay: TemplateChild<adw::ToastOverlay>,

        #[template_child]
        pub(super) main_stack: TemplateChild<gtk::Stack>,

        #[template_child]
        pub(super) welcome_view: TemplateChild<ManuscriptWelcomeView>,

        #[template_child]
        pub(super) editor_view_container: TemplateChild<gtk::Box>,

        #[template_child]
        pub(super) project_layout: TemplateChild<ManuscriptProjectLayout>,

        pub(super) settings: ManuscriptSettings,

        pub(super) document_manager: Rc<DocumentManager>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptWindow {
        const NAME: &'static str = "ManuscriptWindow";
        type Type = super::ManuscriptWindow;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            let mut this = Self::default();
            this.document_manager = Rc::new(DocumentManager::default());

            this
        }

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("win.new-project", None, move |win, _, _| {
                win.new_project();
            });

            klass.install_action("win.open-project", None, move |win, _, _| {
                win.open_project();
            });
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
                win.add_toast(i18n(err));
            }),
        );
    }

    fn restore_window_state(&self) {
        let settings = &self.imp().settings;
        self.set_default_size(settings.window_width(), settings.window_height());
    }

    fn connect_events(&self) {
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
    }

    fn on_document_loaded(&self) {
        // Get a reference to the new document
        let new_document = self
            .imp()
            .document_manager
            .document_ref()
            .lock()
            .expect("Could not lock document");
        let new_document = new_document.deref();

        // Update project layout
        self.imp().project_layout.load_document(new_document);
    }

    fn on_chunk_added(&self, id: String) {
        if let Ok(lock) = self.imp().document_manager.document_ref().lock() {
            let added_chunk = lock.get_chunk_ref(id.as_str()).unwrap().as_ref();

            let view = self.imp().editor_view_container.get();
            let text_view = ManuscriptTextEditor::new();
            text_view.set_halign(gtk::Align::Fill);
            text_view.set_valign(gtk::Align::Fill);
            text_view.set_hexpand(true);
            text_view.init(id, None);
            view.append(&text_view);

            self.imp().project_layout.add_chunk(added_chunk);
        }
    }

    fn on_chunk_removed(&self, id: String) {
        if let Ok(lock) = self.imp().document_manager.document_ref().lock() {
            let removed_chunk = lock.get_chunk_ref(id.as_str()).unwrap().as_ref();
            self.imp().project_layout.remove_chunk(id);
        }
    }

    fn set_document(&self, document: Document) {
        self.imp()
            .document_manager
            .set_document(document)
            .expect("Could not set document");
    }

    fn add_chapter(&self) {
        self.imp().document_manager.add_chunk(Chapter::default());
    }
}
