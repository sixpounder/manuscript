use crate::{
    libs::{consts::*, text_metrics::get_font_size},
    services::analyst::{MarkupHandler, RegexMatch, TagLookup, TEXT_ANALYZER},
};
use gtk::{glib, glib::ToValue, pango, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

const G_LOG_DOMAIN: &str = "ManuscriptBuffer";

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecObject};
    use once_cell::sync::Lazy;

    #[derive(Default)]
    pub struct ManuscriptBuffer {
        pub(super) parent_view: RefCell<Option<gtk::TextView>>,
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
        }

        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "parent-view",
                    "",
                    "",
                    Option::<gtk::TextView>::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "parent-view" => obj.parent_view().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "parent-view" => obj.set_parent_view(value.get::<Option<gtk::TextView>>().unwrap()),
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
        let obj: Self =
            glib::Object::new(&[("tag-table", &tag_table), ("parent-view", &parent_view)]);

        obj.bind_default_tags();
        obj
    }

    pub fn parent_view(&self) -> Option<gtk::TextView> {
        self.imp().parent_view.borrow().clone()
    }

    pub fn set_parent_view(&self, value: Option<gtk::TextView>) {
        *self.imp().parent_view.borrow_mut() = value;
    }

    pub fn format_for(&self, view: &gtk::TextView) {
        let tags = TEXT_ANALYZER.analyze_buffer(self.upcast_ref::<gtk::TextBuffer>(), view);
        self.clear_tags();
        self.apply(tags, view);
    }

    fn clear_tags(&self) {
        let start_iter = self.start_iter();
        let end_iter = self.end_iter();
        self.tag_table()
            .foreach(|tag| self.remove_tag(tag, &start_iter, &end_iter));
    }

    fn apply(&self, tags: Vec<TagLookup>, view: &gtk::TextView) {
        for tag in tags {
            glib::g_debug!(G_LOG_DOMAIN, "Apply tag {:?}", tag);
            match tag {
                TagLookup::ByName(target_tag, start, end) => {
                    if let Some(view_tag) = self.tag_table().lookup(target_tag) {
                        self.apply_tag(
                            &view_tag,
                            &self.iter_at_offset(start),
                            &self.iter_at_offset(end),
                        );
                    } else {
                        glib::g_warning!(G_LOG_DOMAIN, "Tag not supported: {}", target_tag);
                    }
                }
                TagLookup::ByValue(tag, start, end) => {
                    self.apply_tag(&tag, &self.iter_at_offset(start), &self.iter_at_offset(end));
                }
            }
        }
    }

    fn bind_default_tags(&self) {
        let buffer = self;

        buffer
            .create_tag(
                Some(TAG_NAME_ITALIC),
                &[
                    ("weight", &PANGO_WEIGHT_NORMAL),
                    ("style", &pango::Style::Italic),
                ],
            )
            .unwrap();

        buffer
            .create_tag(
                Some(TAG_NAME_BOLD),
                &[
                    ("weight", &PANGO_WEIGHT_BOLD),
                    ("style", &pango::Style::Normal),
                ],
            )
            .unwrap();

        buffer
            .create_tag(
                Some(TAG_NAME_BOLD_ITALIC),
                &[
                    ("weight", &PANGO_WEIGHT_BOLD),
                    ("style", &pango::Style::Italic),
                ],
            )
            .unwrap();

        buffer
            .create_tag(Some(TAG_NAME_STRIKETHROUGH), &[("strikethrough", &true)])
            .unwrap();

        buffer
            .create_tag(
                Some(TAG_NAME_CENTER),
                &[("justification", &gtk::Justification::Center)],
            )
            .unwrap();

        buffer
            .create_tag(
                Some(TAG_NAME_WRAP_NONE),
                &[
                    ("wrap-mode", &gtk::WrapMode::None),
                    ("pixels-above-lines", &0i32),
                    ("pixels-below-lines", &0i32),
                ],
            )
            .unwrap();

        buffer
            .create_tag(
                Some(TAG_NAME_PLAIN_TEXT),
                &[
                    ("weight", &PANGO_WEIGHT_NORMAL),
                    ("style", &pango::Style::Normal),
                    ("strikethrough", &false),
                    ("justification", &gtk::Justification::Left),
                ],
            )
            .unwrap();

        buffer
            .create_tag(
                Some(TAG_NAME_GRAY_TEXT),
                &[
                    ("weight", &PANGO_WEIGHT_NORMAL),
                    ("style", &pango::Style::Normal),
                    ("foreground", &"gray"),
                ],
            )
            .unwrap();

        buffer
            .create_tag(
                Some(TAG_NAME_LINK_COLOR_TEXT),
                &[
                    ("weight", &PANGO_WEIGHT_NORMAL),
                    ("style", &pango::Style::Italic),
                    ("foreground", &"lightblue"),
                ],
            )
            .unwrap();

        buffer
            .create_tag(
                Some(TAG_NAME_UNFOCUSED_TEXT),
                &[
                    ("weight", &PANGO_WEIGHT_NORMAL),
                    ("style", &pango::Style::Normal),
                    ("foreground", &"gray"),
                ],
            )
            .unwrap();

        buffer
            .create_tag(
                Some(TAG_NAME_CODE_TEXT),
                &[
                    ("weight", &PANGO_WEIGHT_NORMAL),
                    ("style", &pango::Style::Normal),
                    ("strikethrough", &false),
                ],
            )
            .unwrap();

        let small_font_size = match self.parent_view().as_ref() {
            Some(view) => (get_font_size(view) as f64 * 0.4).ceil() as i32 * pango::SCALE,
            None => 1000i32,
        };

        buffer
            .create_tag(
                Some(TAG_NAME_SUBSCRIPT),
                &[
                    ("weight", &PANGO_WEIGHT_LIGHT),
                    ("style", &pango::Style::Normal),
                    ("size", &small_font_size), // ("rise", &(pango::units_to_double(-10) as i32)),
                ],
            )
            .unwrap();

        buffer
            .create_tag(
                Some(TAG_NAME_SUPERSCRIPT),
                &[
                    ("weight", &PANGO_WEIGHT_LIGHT),
                    ("style", &pango::Style::Normal),
                    ("size", &small_font_size), // ("rise", &(pango::units_to_double(-1000) as i32)),
                ],
            )
            .unwrap();

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
