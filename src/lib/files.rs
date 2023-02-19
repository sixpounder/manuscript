use crate::{config::G_LOG_DOMAIN, models::*, services::i18n::i18n};
use adw::subclass::prelude::*;
use gtk::gio;
use gtk::prelude::*;
use std::io::Read;

pub fn with_file_open_dialog<F, E>(on_success: F, on_error: E)
where
    F: Fn(Document) + 'static,
    E: Fn(&'static str) + 'static,
{
    let app = gio::Application::default()
        .expect("Failed to retrieve application singleton")
        .downcast::<gtk::Application>()
        .unwrap();
    let win = app
        .active_window()
        .unwrap()
        .downcast::<gtk::Window>()
        .unwrap();

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
                                    on_success(document);
                                },
                                Err(error) => {
                                    on_error("Unreadable file");

                                }
                            }
                        } else {
                            // Failed to read file
                            on_error("Unreadable file");
                        }
                    } else {
                        // File not accessible
                        on_error("File not existing or not accessible");
                    }
                }
            }
        }
    }));

    dialog.show();
}
