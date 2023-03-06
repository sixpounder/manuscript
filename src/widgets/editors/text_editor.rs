use crate::{
    models::*,
    services::{prelude::bytes_from_text_buffer, DocumentAction, BufferStats, i18n::i18n},
    widgets::ManuscriptProgressIndicator,
};
use adw::subclass::prelude::*;
use bytes::Bytes;
use gtk::{
    gio, glib,
    glib::{clone, Sender},
    prelude::*,
};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/editors/text_editor.ui")]
    pub struct ManuscriptTextEditor {
        #[template_child]
        pub(super) scroll_container: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub(super) text_view: TemplateChild<gtk::TextView>,

        #[template_child]
        pub(super) progress_indicator: TemplateChild<ManuscriptProgressIndicator>,

        #[template_child]
        pub(super) words_count_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub(super) reading_time_label: TemplateChild<gtk::Label>,

        pub(super) sender: RefCell<Option<Sender<DocumentAction>>>,
        pub(super) chunk_id: RefCell<Option<String>>,
        pub(super) text_buffer: RefCell<Option<gtk::TextBuffer>>,
        pub(super) update_idle_resource_id: RefCell<Option<glib::SourceId>>,
        pub(super) locked: Cell<bool>,
        pub(super) words_count: Cell<u64>,
        pub(super) reading_time: Cell<(u64, u64)>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptTextEditor {
        const NAME: &'static str = "ManuscriptTextEditor";
        type Type = super::ManuscriptTextEditor;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptTextEditor {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_widgets();
        }

        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecBoolean::new("locked", "", "", false, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("overflowing", "", "", false, ParamFlags::READABLE),
                    ParamSpecString::new("chunk-id", "", "", None, ParamFlags::READWRITE),
                    ParamSpecString::new("words-count-label-text", "", "", None, ParamFlags::READABLE),
                    ParamSpecString::new("reading-time-label-text", "", "", None, ParamFlags::READABLE),
                    ParamSpecObject::new(
                        "buffer",
                        "",
                        "",
                        Option::<gtk::TextBuffer>::static_type(),
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            let imp = obj.imp();
            match pspec.name() {
                "locked" => imp.locked.get().to_value(),
                "chunk-id" => imp.chunk_id.borrow().to_value(),
                "buffer" => imp.text_buffer.borrow().to_value(),
                "words-count-label-text" => {
                    let words_count = imp.words_count.get();
                    format!("{} {}", words_count, i18n("words")).to_value()
                },
                "reading-time-label-text" => {
                    let reading_time = imp.reading_time.get();
                    format!("{} {}", reading_time.0, i18n("minutes")).to_value()
                },
                "overflowing" => {
                    let adjustment = imp.scroll_container.vadjustment();
                    (adjustment.upper() > (adjustment.lower() + adjustment.page_size())).to_value()
                },
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let _obj = self.obj();
            match pspec.name() {
                "locked" => self.locked.set(value.get::<bool>().unwrap()),
                "chunk-id" => {
                    self.chunk_id
                        .replace(value.get::<Option<String>>().unwrap());
                }
                "buffer" => {
                    self.text_buffer
                        .replace(value.get::<Option<gtk::TextBuffer>>().unwrap());
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ManuscriptTextEditor {}
}

glib::wrapper! {
    pub struct ManuscriptTextEditor(ObjectSubclass<imp::ManuscriptTextEditor>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ManuscriptTextEditor {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ManuscriptTextEditor {
    pub fn new(sender: Option<Sender<DocumentAction>>) -> Self {
        let obj: Self = glib::Object::new(&[]);
        obj.imp().sender.replace(sender);

        obj
    }

    pub fn init(&self, chunk_id: String, buffer: Option<Bytes>) {
        let imp = self.imp();

        imp.chunk_id.replace(Some(chunk_id));
        self.set_buffer(buffer);
    }

    fn setup_widgets(&self) {
        let imp = self.imp();
        imp.scroll_container.vadjustment().connect_value_changed(
            glib::clone!(@weak self as this => move |adjustment| {
                let text_view_allocation = this.imp().text_view.allocation();
                let progress_indicator = this.imp().progress_indicator.get();
                progress_indicator.set_value(adjustment.value().floor() as i32);
                progress_indicator.set_minimum(adjustment.lower().floor() as i32);
                progress_indicator.set_maximum(adjustment.upper().floor() as i32 - text_view_allocation.height());
                this.notify("overflowing");
            })
        );
    }

    fn set_buffer(&self, value: Option<Bytes>) {
        let imp = self.imp();
        let text_buffer = gtk::TextBuffer::new(None);
        let bytes = value.unwrap_or(Bytes::new());
        imp.words_count.set(bytes.words_count());
        imp.reading_time.set(bytes.estimate_reading_time());

        text_buffer.set_text(
            String::from_utf8(bytes.slice(..).to_vec())
                .unwrap()
                .as_str(),
        );
        imp.text_view.set_buffer(Some(&text_buffer));
        imp.text_buffer.replace(Some(text_buffer));
        self.connect_text_buffer();
        self.notify("overflowing");
    }

    pub fn text_buffer(&self) -> std::cell::Ref<Option<gtk::TextBuffer>> {
        self.imp().text_buffer.borrow()
    }

    fn connect_text_buffer(&self) {
        if let Some(buffer) = self.text_buffer().as_ref() {
            buffer.connect_changed(clone!(@strong self as this => move |buf| {
                this.on_buffer_changed(buf);
            }));
        }
    }

    pub fn words_count(&self) -> u64 {
        self.imp().words_count.get()
    }

    pub fn set_words_count(&self, value: u64) {
        self.imp().words_count.set(value);
        self.notify("words-count-label-text");
    }

    pub fn reading_time(&self) -> (u64, u64) {
        self.imp().reading_time.get()
    }

    pub fn set_reading_time(&self, value: (u64, u64)) {
        self.imp().reading_time.set(value);
        self.notify("reading-time-label-text");
    }

    fn on_buffer_changed(&self, _buffer: &gtk::TextBuffer) {
        let imp = self.imp();
        let chunk_id = imp.chunk_id.borrow();

        if let Some(chunk_id) = chunk_id.as_ref() {
            if let Some(buf) = self.text_buffer().as_ref() {
                let bytes = bytes_from_text_buffer(buf);
                let tx = imp.sender.borrow();
                let tx = tx.as_ref().expect("No channel sender found");
                tx.send(DocumentAction::UpdateChunkBuffer(
                    chunk_id.to_string(),
                    bytes,
                )).expect("Could not send buffer updates");
                // TODO: instead of expecting this value, handle failures graphically

                self.notify("overflowing");
            }

            // Cancel any closure registered before, obtain a debounce effect
            let mut source_id = self.imp().update_idle_resource_id.borrow_mut();
            if source_id.is_some() {
                let source_id = source_id.take().unwrap();
                source_id.remove();
            }
            drop(source_id);

            let source_id = glib::source::timeout_add_seconds_local(
                1,
                glib::clone!(@weak self as this => @default-return glib::Continue(false), move || {
                    if let Some(buf) = this.text_buffer().as_ref() {
                        let imp = this.imp();

                        let bytes = bytes_from_text_buffer(buf);
                        let words_count = bytes.words_count();
                        let (reading_time_minutes, reading_time_seconds) = bytes.estimate_reading_time();
                        this.set_words_count(words_count);
                        this.set_reading_time((reading_time_minutes, reading_time_seconds));

                        let chunk_id = imp.chunk_id.borrow();
                            if let Some(chunk_id) = chunk_id.as_ref() {
                            let tx = imp.sender.borrow();
                            let tx = tx.as_ref().expect("No channel sender found");

                            // Ignore any error here, as this is non blocking and will result
                            // in "only" a UI inconsistency
                            let _ = tx.send(DocumentAction::UpdateChunkBufferStats(
                                chunk_id.to_string(),
                                BufferStats::new(words_count, (reading_time_minutes, reading_time_seconds))
                            ));
                        }
                        imp.update_idle_resource_id.replace(None);
                    }

                    glib::Continue(false)
                }),
            );
            self.imp().update_idle_resource_id.replace(Some(source_id));
        } else {
            panic!("No chunk id is set on this ManuscriptTextEditor. This is suspicious so I am going to kill everything.");
        }
    }
}
