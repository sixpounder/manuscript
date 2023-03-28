use super::ManuscriptBuffer;
use crate::{
    models::*,
    services::{
        i18n::i18n, prelude::bytes_from_text_buffer, BufferStats, DocumentAction,
        ManuscriptSettings,
    },
    widgets::ManuscriptProgressIndicator,
};
use adw::subclass::prelude::*;
use bytes::Bytes;
use gtk::{
    gio, glib,
    glib::{clone, closure_local, ParamSpec, Sender},
    prelude::*,
};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;
    use glib::{ParamSpecBoolean, ParamSpecInt, ParamSpecObject, ParamSpecString};
    use once_cell::sync::Lazy;

    #[derive(gtk::CompositeTemplate)]
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
        pub(super) text_buffer: RefCell<Option<ManuscriptBuffer>>,
        pub(super) metrics_idle_resource_id: RefCell<Option<glib::SourceId>>,
        pub(super) format_idle_resource_id: RefCell<Option<glib::SourceId>>,
        pub(super) locked: Cell<bool>,
        pub(super) show_status_bar: Cell<bool>,
        pub(super) words_count: Cell<u64>,
        pub(super) reading_time: Cell<(u64, u64)>,
        pub(super) settings: ManuscriptSettings,
        pub(super) style_manager: adw::StyleManager,
    }

    impl Default for ManuscriptTextEditor {
        fn default() -> Self {
            Self {
                scroll_container: TemplateChild::default(),
                text_view: TemplateChild::default(),
                progress_indicator: TemplateChild::default(),
                words_count_label: TemplateChild::default(),
                reading_time_label: TemplateChild::default(),
                sender: RefCell::default(),
                text_buffer: RefCell::default(),
                metrics_idle_resource_id: RefCell::default(),
                format_idle_resource_id: RefCell::default(),
                chunk_id: RefCell::new(None),
                show_status_bar: Cell::new(true),
                locked: Cell::new(false),
                words_count: Cell::new(0),
                reading_time: Cell::new((0, 0)),
                settings: ManuscriptSettings::default(),
                style_manager: adw::StyleManager::default(),
            }
        }
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
            let obj = self.obj();
            obj.setup_widgets();
            obj.setup_themes();
        }

        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecBoolean::builder("show-status-bar")
                        .default_value(false)
                        .readwrite()
                        .build(),
                    ParamSpecBoolean::builder("locked")
                        .default_value(false)
                        .readwrite()
                        .build(),
                    ParamSpecBoolean::builder("overflowing")
                        .default_value(false)
                        .readwrite()
                        .build(),
                    ParamSpecString::builder("chunk-id")
                        .default_value(None)
                        .read_only()
                        .build(),
                    ParamSpecString::builder("words-count-label-text")
                        .default_value(None)
                        .read_only()
                        .build(),
                    ParamSpecString::builder("reading-time-label-text")
                        .default_value(None)
                        .read_only()
                        .build(),
                    ParamSpecObject::builder::<gtk::TextBuffer>("buffer")
                        .readwrite()
                        .build(),
                    ParamSpecInt::builder("paragraph-spacing")
                        .minimum(0)
                        .maximum(100)
                        .default_value(36)
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            let imp = obj.imp();
            match pspec.name() {
                "show-status-bar" => imp.show_status_bar.get().to_value(),
                "locked" => imp.locked.get().to_value(),
                "chunk-id" => imp.chunk_id.borrow().to_value(),
                "buffer" => imp.text_buffer.borrow().to_value(),
                "words-count-label-text" => {
                    let words_count = imp.words_count.get();
                    format!("{} {}", words_count, i18n("words")).to_value()
                }
                "reading-time-label-text" => {
                    let reading_time = imp.reading_time.get();
                    format!("~ {} {}", reading_time.0, i18n("minutes")).to_value()
                }
                "overflowing" => {
                    let adjustment = imp.scroll_container.vadjustment();
                    (adjustment.upper() > (adjustment.lower() + adjustment.page_size())).to_value()
                }
                "paragraph-spacing" => 36i32.to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let _obj = self.obj();
            match pspec.name() {
                "show-status-bar" => self.show_status_bar.set(value.get::<bool>().unwrap()),
                "locked" => self.locked.set(value.get::<bool>().unwrap()),
                "chunk-id" => {
                    self.chunk_id
                        .replace(value.get::<Option<String>>().unwrap());
                }
                "buffer" => {
                    self.text_buffer
                        .replace(value.get::<Option<ManuscriptBuffer>>().unwrap());
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
        let obj: Self = glib::Object::new();
        obj.imp().sender.replace(sender);

        obj
    }

    pub fn init(&self, chunk_id: String, buffer: Option<Bytes>) {
        let imp = self.imp();

        imp.chunk_id.replace(Some(chunk_id));
        self.set_buffer_irreversible(buffer);
    }

    pub fn chunk_id(&self) -> Option<String> {
        let chunk_id = self.imp().chunk_id.borrow();
        chunk_id.as_ref().map(|c| c.to_string())
    }

    fn setup_widgets(&self) {
        let imp = self.imp();

        imp.style_manager
            .connect_dark_notify(glib::clone!(@weak self as this => move |_| {
                this.setup_themes();
            }));

        imp.scroll_container.vadjustment().connect_value_changed(
            glib::clone!(@weak self as this => move |adjustment| {
                let imp = this.imp();
                let text_view_allocation = imp.text_view.allocation();
                let progress_indicator = imp.progress_indicator.get();
                progress_indicator.set_value(adjustment.value().floor() as i32);
                progress_indicator.set_minimum(adjustment.lower().floor() as i32);
                progress_indicator.set_maximum(adjustment.upper().floor() as i32 - text_view_allocation.height());
                this.notify("overflowing");
            })
        );
    }

    fn setup_themes(&self) {
        if let Some(buf) = self.text_buffer().as_ref() {
            buf.reload_colors(&self.style_context());
        }
    }

    fn set_buffer_irreversible(&self, value: Option<Bytes>) {
        self.set_buffer(value, true);
        self.debounce_analyze();
    }

    fn set_buffer(&self, value: Option<Bytes>, irreversible: bool) {
        let text_buffer = ManuscriptBuffer::new(None, Some(self.text_view()));
        let bytes = value.unwrap_or(Bytes::new());
        let imp = self.imp();
        imp.words_count.set(bytes.words_count());
        imp.reading_time.set(bytes.estimate_reading_time());

        if irreversible {
            text_buffer.begin_irreversible_action();
            text_buffer.set_text(
                String::from_utf8(bytes.slice(..).to_vec())
                    .unwrap()
                    .as_str(),
            );
            text_buffer.end_irreversible_action();
        } else {
            text_buffer.set_text(
                String::from_utf8(bytes.slice(..).to_vec())
                    .unwrap()
                    .as_str(),
            );
        }
        imp.text_view.set_buffer(Some(&text_buffer));
        imp.text_buffer.replace(Some(text_buffer));
        self.connect_text_buffer();
        self.setup_themes();
        self.idle_format();
        self.notify("overflowing");
    }

    pub fn text_buffer(&self) -> std::cell::Ref<Option<ManuscriptBuffer>> {
        self.imp().text_buffer.borrow()
    }

    pub fn text_buffer_mut(&self) -> std::cell::RefMut<Option<ManuscriptBuffer>> {
        self.imp().text_buffer.borrow_mut()
    }

    pub fn text_view(&self) -> gtk::TextView {
        self.imp().text_view.get()
    }

    pub fn settings(&self) -> &ManuscriptSettings {
        &self.imp().settings
    }

    fn connect_text_buffer(&self) {
        if let Some(buffer) = self.text_buffer().as_ref() {
            buffer.connect_changed(clone!(@strong self as this => move |buf| {
                this.on_buffer_changed(buf);
            }));

            buffer.connect_closure(
                "parse-first-header",
                false,
                closure_local!(@strong self as this => move |_buf: ManuscriptBuffer, spec: String| {
                    this.on_parse_first_header(spec);
                }),
            );
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

    pub fn locked(&self) -> bool {
        self.imp().locked.get()
    }

    pub fn set_locked(&self, value: bool) {
        let imp = self.imp();
        imp.locked.set(value);
        self.send_update(move |chunk| {
            chunk.set_locked(value);
        });
    }

    fn on_buffer_changed(&self, _buffer: &ManuscriptBuffer) {
        let imp = self.imp();
        let chunk_id = imp.chunk_id.borrow();

        if let Some(chunk_id) = chunk_id.as_ref() {
            if let Some(buf) = self.text_buffer().as_ref() {
                let bytes = bytes_from_text_buffer(buf.upcast_ref::<gtk::TextBuffer>());
                let tx = imp.sender.borrow();
                let tx = tx.as_ref().expect("No channel sender found");
                tx.send(DocumentAction::UpdateChunkBuffer(
                    chunk_id.to_string(),
                    bytes,
                ))
                .expect("Could not send buffer updates");
                // TODO: instead of expecting this value, handle failures graphically

                self.idle_format();

                self.notify("overflowing");
            }

            self.debounce_analyze();
        } else {
            panic!("No chunk id is set on this ManuscriptTextEditor. This is suspicious so I am going to kill everything 🤷‍♂️️.");
        }
    }

    fn on_parse_first_header(&self, value: String) {
        self.send_update(move |chunk| {
            if let Some(chapter) = chunk.as_any_mut().downcast_mut::<Chapter>() {
                chapter.set_title(value.as_str());
            }
        });
    }

    fn debounce_analyze(&self) {
        // Cancel any closure registered before, obtain a debounce effect
        let mut source_id = self.imp().metrics_idle_resource_id.borrow_mut();
        if source_id.is_some() {
            let source_id = source_id.take().unwrap();
            source_id.remove();
        }
        drop(source_id);

        let delay = std::time::Duration::from_millis(
            self.settings().text_analysis_delay().unsigned_abs() as u64,
        );
        let source_id = glib::source::timeout_add_local(
            delay,
            glib::clone!(@weak self as this => @default-return glib::Continue(false), move || {
                let imp = this.imp();
                if let Some(buf) = this.text_buffer().as_ref() {
                    let bytes = bytes_from_text_buffer(buf.upcast_ref::<gtk::TextBuffer>());
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
                }

                *imp.metrics_idle_resource_id.borrow_mut() = None;
                glib::Continue(false)
            }),
        );
        *self.imp().metrics_idle_resource_id.borrow_mut() = Some(source_id);
    }

    fn idle_format(&self) {
        let mut source_id = self.imp().format_idle_resource_id.borrow_mut();
        if source_id.is_some() {
            let source_id = source_id.take().unwrap();
            source_id.remove();
        }
        drop(source_id);

        let source_id =
            glib::source::idle_add_local_once(glib::clone!(@strong self as this => move || {
                let imp = this.imp();
                if let Some(buf) = this.text_buffer().as_ref() {
                    buf.format_for(&this.text_view());
                }
                *imp.format_idle_resource_id.borrow_mut() = None;
            }));
        *self.imp().format_idle_resource_id.borrow_mut() = Some(source_id);
    }

    fn send_update<F>(&self, f: F)
    where
        F: FnOnce(&mut dyn DocumentChunk) + 'static,
    {
        let chunk_id = self.chunk_id().expect("No chunk id");
        self.sender()
            .send(DocumentAction::UpdateChunkWith(chunk_id, Box::new(f)))
            .expect("Failed to send character sheet update");
    }

    fn sender(&self) -> Sender<DocumentAction> {
        let tx = self.imp().sender.borrow();
        tx.as_ref().expect("No channel sender found").clone()
    }
}
