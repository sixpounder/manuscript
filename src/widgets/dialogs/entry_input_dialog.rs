use adw::{
    prelude::{MessageDialogExt, PreferencesRowExt},
    subclass::prelude::*,
};
use gtk::{gio, glib, prelude::*};

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/dialogs/entry_input_dialog.ui")]
    pub struct ManuscriptEntryInputDialog {
        pub(super) entry_row: adw::EntryRow,
    }

    impl Default for ManuscriptEntryInputDialog {
        fn default() -> Self {
            Self {
                entry_row: adw::EntryRow::builder().build(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptEntryInputDialog {
        const NAME: &'static str = "ManuscriptEntryInputDialog";
        type Type = super::ManuscriptEntryInputDialog;
        type ParentType = adw::MessageDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptEntryInputDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup();
        }

        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("heading")
                        .flags(ParamFlags::READWRITE)
                        .build(),
                    ParamSpecString::builder("body")
                        .flags(ParamFlags::READWRITE)
                        .build(),
                    ParamSpecString::builder("entry-label")
                        .flags(ParamFlags::READWRITE)
                        .build(),
                    ParamSpecString::builder("entry-text")
                        .flags(ParamFlags::READWRITE)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "heading" => obj.heading().to_value(),
                "body" => obj.body().to_value(),
                "entry-label" => obj.entry_row().title().to_value(),
                "entry-text" => obj.entry_row().text().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "heading" => obj.set_heading(Some(value.get::<String>().unwrap().as_str())),
                "body" => obj.set_body(value.get::<String>().unwrap().as_str()),
                "entry-label" => obj
                    .entry_row()
                    .set_title(value.get::<String>().unwrap().as_str()),
                "entry-text" => obj
                    .entry_row()
                    .set_text(value.get::<String>().unwrap().as_str()),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ManuscriptEntryInputDialog {}
    impl WindowImpl for ManuscriptEntryInputDialog {}
    impl MessageDialogImpl for ManuscriptEntryInputDialog {}
}

glib::wrapper! {
    pub struct ManuscriptEntryInputDialog(ObjectSubclass<imp::ManuscriptEntryInputDialog>)
        @extends adw::MessageDialog, gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptEntryInputDialog {
    pub fn new(parent: &gtk::Window, args: &[(&str, &dyn ToValue)]) -> Self {
        let mut args = args.to_vec();
        args.push(("modal", &true));
        args.push(("transient-for", parent));
        let mut builder = glib::Object::builder();
        for arg in args {
            builder = builder.property(arg.0, arg.1);
        }

        builder.build()
    }

    fn setup(&self) {
        let entry_row = &self.imp().entry_row;
        self.bind_property("entry-label", entry_row, "title")
            .bidirectional()
            .sync_create()
            .build();
        self.bind_property("entry-text", entry_row, "text")
            .bidirectional()
            .sync_create()
            .build();

        let listbox = gtk::ListBox::builder()
            .css_classes(vec!["boxed-list"])
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        listbox.append(entry_row);
        listbox.set_width_request(320);

        self.set_extra_child(Some(&listbox));
    }

    fn entry_row(&self) -> adw::EntryRow {
        adw::EntryRow::builder().build()
    }
}
