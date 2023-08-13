use crate::services::i18n::i18n;
use glib;
use gtk::gio;
use gtk::prelude::*;

/// Gets the currently active window for
/// this application
pub fn window() -> gtk::Window {
    let app = gio::Application::default()
        .expect("Failed to retrieve application singleton")
        .downcast::<gtk::Application>()
        .unwrap();
    app.active_window()
        .unwrap()
        .downcast::<gtk::Window>()
        .unwrap()
}

/// Shows a file selection dialog for manuscripts
/// and executes `on_done` when a file is selected
pub fn with_file_open_dialog<F>(on_done: F)
where
    F: Fn(String) + 'static,
{
    let win = window();
    let dialog = gtk::FileChooserNative::builder()
        .accept_label(i18n("_Open"))
        .cancel_label(i18n("_Cancel"))
        .modal(true)
        .title(i18n("Open manuscript"))
        .transient_for(&win)
        .select_multiple(false)
        .action(gtk::FileChooserAction::Open)
        .build();

    let manuscript_file_filter = gtk::FileFilter::new();
    manuscript_file_filter.set_name(Some(&i18n("Manuscript files")));
    manuscript_file_filter.add_mime_type("application/x-manuscript");

    let any_file_filter = gtk::FileFilter::new();
    any_file_filter.set_name(Some(&i18n("All files")));
    any_file_filter.add_pattern("*");

    dialog.add_filter(&manuscript_file_filter);
    dialog.add_filter(&any_file_filter);

    dialog.connect_response(glib::clone!(@strong dialog => move |_, response| {
        let file = dialog.file();
        if response == gtk::ResponseType::Accept {
            if let Some(file) = file.as_ref() {
                if file.query_exists(gio::Cancellable::NONE) {
                    let file_io_stream = dialog.file().unwrap();
                    let file_name = file_io_stream.path().unwrap();
                    let file_name = file_name.to_str().unwrap();

                    on_done(file_name.into());
                }
            }
        }
    }));

    dialog.show();
}

/// Shows a file selection dialog for manuscripts
/// with a save intent and executes `on_choice` when a file is selected
pub fn with_file_save_dialog<F>(on_choice: F)
where
    F: Fn(String) + 'static,
{
    let win = window();

    let dialog = gtk::FileChooserNative::builder()
        .accept_label(i18n("_Save"))
        .cancel_label(i18n("_Cancel"))
        .modal(true)
        .title(i18n("Save manuscript"))
        .transient_for(&win)
        .select_multiple(false)
        .action(gtk::FileChooserAction::Save)
        .build();

    let manuscript_file_filter = gtk::FileFilter::new();
    manuscript_file_filter.set_name(Some(&i18n("Manuscript files")));
    manuscript_file_filter.add_pattern("*.mscript");

    let any_file_filter = gtk::FileFilter::new();
    any_file_filter.set_name(Some(&i18n("All files")));
    any_file_filter.add_mime_type("application/x-manuscript");
    any_file_filter.add_pattern("*");

    dialog.add_filter(&manuscript_file_filter);
    dialog.add_filter(&any_file_filter);

    dialog.connect_response(glib::clone!(@strong dialog => move |_, response| {
        let file = dialog.file();
        if response == gtk::ResponseType::Accept {
            if let Some(file) = file.as_ref() {
                on_choice(file.path().unwrap().to_str().unwrap().into());
            }
        }
    }));

    dialog.show();
}
