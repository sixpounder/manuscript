use crate::{
    libs::{consts::*, text_metrics::get_font_size},
    services::analyst::{MarkupHandler, RegexMatch, TagApplyRules, TagLookup, TEXT_ANALYZER},
};
use gtk::{glib, glib::ToValue, pango, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};

const G_LOG_DOMAIN: &str = "ManuscriptBuffer";

mod imp {
    use super::*;
    use glib::{
        subclass::signal::Signal, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecObject,
    };
    use once_cell::sync::Lazy;

    #[derive(Default)]
    pub struct ManuscriptBuffer {
        pub(super) parent_view: RefCell<Option<gtk::TextView>>,
        pub(super) matched_tags: RefCell<Vec<TagApplyRules>>,
        pub(super) autoformat: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptBuffer {
        const NAME: &'static str = "ManuscriptBuffer";
        type Type = super::ManuscriptBuffer;
        type ParentType = gtk::TextBuffer;
    }

    impl ObjectImpl for ManuscriptBuffer {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().connect_events();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("parsed").build(),
                    Signal::builder("parse-first-header")
                        .param_types([String::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "parent-view",
                        "",
                        "",
                        Option::<gtk::TextView>::static_type(),
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecBoolean::new("autoformat", "", "", true, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "parent-view" => obj.parent_view().to_value(),
                "autoformat" => obj.autoformat().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "parent-view" => obj.set_parent_view(value.get::<Option<gtk::TextView>>().unwrap()),
                "autoformat" => obj.set_autoformat(value.get::<bool>().unwrap()),
                _ => unimplemented!(),
            }
        }
    }
    impl TextBufferImpl for ManuscriptBuffer {}
}

glib::wrapper! {
    pub struct ManuscriptBuffer(ObjectSubclass<imp::ManuscriptBuffer>) @extends gtk::TextBuffer;
}

impl Default for ManuscriptBuffer {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl ManuscriptBuffer {
    pub fn new(tag_table: Option<gtk::TextTagTable>, parent_view: Option<gtk::TextView>) -> Self {
        glib::Object::new(&[("tag-table", &tag_table), ("parent-view", &parent_view)])
    }

    fn connect_events(&self) {
        self.connect_changed(glib::clone!(@strong self as this => move |_| {
            if this.imp().autoformat.get() {
                this.format();
            }
        }));
    }

    pub fn autoformat(&self) -> bool {
        self.imp().autoformat.get()
    }

    pub fn set_autoformat(&self, value: bool) {
        self.imp().autoformat.set(value);
    }

    pub fn parent_view(&self) -> Option<gtk::TextView> {
        self.imp().parent_view.borrow().clone()
    }

    fn set_parent_view(&self, value: Option<gtk::TextView>) {
        *self.imp().parent_view.borrow_mut() = value;
        self.bind_default_tags();
    }

    pub fn parsed_tags(&self) -> std::cell::Ref<Vec<TagApplyRules>> {
        self.imp().matched_tags.borrow()
    }

    pub fn format_for(&self, view: &gtk::TextView) {
        let tags = TEXT_ANALYZER.analyze_buffer(self.upcast_ref::<gtk::TextBuffer>(), view);
        self.clear_tags();
        self.apply(tags, view);
    }

    pub fn format(&self) {
        if let Some(view) = self.parent_view() {
            self.format_for(&view);
        }
    }

    fn clear_tags(&self) {
        let start_iter = self.start_iter();
        let end_iter = self.end_iter();
        self.remove_all_tags(&start_iter, &end_iter);
    }

    fn apply(&self, rules: Vec<TagApplyRules>, view: &gtk::TextView) {
        let rules_iter = rules.clone();
        *self.imp().matched_tags.borrow_mut() = rules;
        let mut maybe_first_title: Option<String> = None;

        for rule in rules_iter {
            if maybe_first_title.is_none() && rule.is_header() {
                let header_candidate = rule.header_candidate().unwrap().clone();
                maybe_first_title = Some(header_candidate.clone());
                self.emit_by_name::<()>("parse-first-header", &[&header_candidate]);
            }

            glib::g_debug!(G_LOG_DOMAIN, "Apply rule {:?}", rule);
            for tag in rule.rules() {
                match tag {
                    TagLookup::ByName(target_tag, start, end) => {
                        if let Some(view_tag) = self.tag_table().lookup(target_tag) {
                            self.apply_tag(
                                &view_tag,
                                &self.iter_at_offset(*start),
                                &self.iter_at_offset(*end),
                            );
                        } else {
                            glib::g_warning!(G_LOG_DOMAIN, "Tag not supported: {}", target_tag);
                        }
                    }
                    TagLookup::ByValue(tag, start, end) => {
                        self.apply_tag(
                            tag,
                            &self.iter_at_offset(*start),
                            &self.iter_at_offset(*end),
                        );
                    }
                }
            }
        }
        self.emit_by_name::<()>("parsed", &[]);
    }

    fn bind_default_tags(&self) {
        let buffer = self;

        let _ = buffer.create_tag(
            Some(TAG_NAME_ITALIC),
            &[
                ("weight", &PANGO_WEIGHT_NORMAL),
                ("style", &pango::Style::Italic),
            ],
        );

        let _ = buffer.create_tag(
            Some(TAG_NAME_BOLD),
            &[
                ("weight", &PANGO_WEIGHT_BOLD),
                ("style", &pango::Style::Normal),
            ],
        );

        let _ = buffer.create_tag(
            Some(TAG_NAME_BOLD_ITALIC),
            &[
                ("weight", &PANGO_WEIGHT_BOLD),
                ("style", &pango::Style::Italic),
            ],
        );

        let _ = buffer.create_tag(Some(TAG_NAME_STRIKETHROUGH), &[("strikethrough", &true)]);

        let _ = buffer.create_tag(
            Some(TAG_NAME_CENTER),
            &[("justification", &gtk::Justification::Center)],
        );

        let _ = buffer.create_tag(
            Some(TAG_NAME_WRAP_NONE),
            &[
                ("wrap-mode", &gtk::WrapMode::None),
                ("pixels-above-lines", &0i32),
                ("pixels-below-lines", &0i32),
            ],
        );

        let _ = buffer.create_tag(
            Some(TAG_NAME_PLAIN_TEXT),
            &[
                ("weight", &PANGO_WEIGHT_NORMAL),
                ("style", &pango::Style::Normal),
                ("strikethrough", &false),
                ("justification", &gtk::Justification::Left),
            ],
        );

        let _ = buffer.create_tag(
            Some(TAG_NAME_GRAY_TEXT),
            &[
                ("weight", &PANGO_WEIGHT_NORMAL),
                ("style", &pango::Style::Normal),
                ("foreground", &"gray"),
            ],
        );

        let _ = buffer.create_tag(
            Some(TAG_NAME_LINK_COLOR_TEXT),
            &[
                ("weight", &PANGO_WEIGHT_NORMAL),
                ("style", &pango::Style::Italic),
                ("foreground", &"lightblue"),
            ],
        );

        let _ = buffer.create_tag(
            Some(TAG_NAME_UNFOCUSED_TEXT),
            &[
                ("weight", &PANGO_WEIGHT_NORMAL),
                ("style", &pango::Style::Normal),
                ("foreground", &"gray"),
            ],
        );

        let _ = buffer.create_tag(
            Some(TAG_NAME_CODE_TEXT),
            &[
                ("weight", &PANGO_WEIGHT_NORMAL),
                ("style", &pango::Style::Normal),
                ("strikethrough", &false),
            ],
        );

        let small_font_size = match self.parent_view().as_ref() {
            Some(view) => (get_font_size(view) as f64 * 0.4).ceil() as i32 * pango::SCALE,
            None => 1000i32,
        };

        let _ = buffer.create_tag(
            Some(TAG_NAME_SUBSCRIPT),
            &[
                ("weight", &PANGO_WEIGHT_LIGHT),
                ("style", &pango::Style::Normal),
                ("size", &small_font_size), // ("rise", &(pango::units_to_double(-10) as i32)),
            ],
        );

        let _ = buffer.create_tag(
            Some(TAG_NAME_SUPERSCRIPT),
            &[
                ("weight", &PANGO_WEIGHT_LIGHT),
                ("style", &pango::Style::Normal),
                ("size", &small_font_size), // ("rise", &(pango::units_to_double(-1000) as i32)),
            ],
        );

        // buffer
        //     .create_tag(
        //         Some(TAG_NAME_CODE_BLOCK),
        //         &[
        //             ("weight", &PANGO_WEIGHT_NORMAL),
        //             ("style", &pango::Style::Normal),
        //             ("strikethrough", &false),
        //             ("indent", &get_margin_indent(self, 0, 1, None, None).1),
        //         ],
        //     )
        //     .unwrap();
    }
}
