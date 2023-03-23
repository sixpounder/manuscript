use crate::{config::G_LOG_DOMAIN, models::*, services::i18n::i18n};
use glib;
use gtk::gio;
use gtk::prelude::*;
use std::io::Read;

fn window() -> gtk::Window {
    let app = gio::Application::default()
        .expect("Failed to retrieve application singleton")
        .downcast::<gtk::Application>()
        .unwrap();
    app.active_window()
        .unwrap()
        .downcast::<gtk::Window>()
        .unwrap()
}

pub fn with_file_open_dialog<F, E>(on_success: F, on_error: E)
where
    F: Fn(String, Document) + 'static,
    E: Fn(String) + 'static,
{
    let win = window();
    let dialog = gtk::FileChooserNative::builder()
        .accept_label(&i18n("_Open"))
        .cancel_label(&i18n("_Cancel"))
        .modal(true)
        .title(&i18n("Open manuscript"))
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
                    let mut buffer: Vec<u8> = vec![];

                    let file_io_stream = dialog.file().unwrap();
                    let file_name = file_io_stream.path().unwrap();
                    let file_name = file_name.to_str().unwrap();

                    if let Ok(file) = std::fs::File::open(file_name) {
                        let mut file = std::io::BufReader::new(file);
                        if let Ok(bytes_read) = file.read_to_end(&mut buffer) {
                            glib::debug!("Opening project (read {} bytes)", bytes_read);

                            match Document::try_from(buffer.as_slice()) {
                                Ok(document) => {
                                    on_success(file_name.into(), document);
                                },
                                Err(_error) => {
                                    on_error(i18n("Unreadable file"));

                                }
                            }
                        } else {
                            // Failed to read file
                            on_error(i18n("Unreadable file"));
                        }
                    } else {
                        // File not accessible
                        on_error(i18n("File not existing or not accessible"));
                    }
                }
            }
        }
    }));

    dialog.show();
}

pub fn with_file_save_dialog<F>(on_choice: F)
where
    F: Fn(String) + 'static,
{
    let win = window();

    let dialog = gtk::FileChooserNative::builder()
        .accept_label(&i18n("_Save"))
        .cancel_label(&i18n("_Cancel"))
        .modal(true)
        .title(&i18n("Save manuscript"))
        .transient_for(&win)
        .select_multiple(false)
        .action(gtk::FileChooserAction::Save)
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
                on_choice(file.path().unwrap().to_str().unwrap().into());
            }
        }
    }));

    dialog.show();
}
