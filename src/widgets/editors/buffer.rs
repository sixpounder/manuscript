use crate::{
    libs::consts::*,
    services::analyst::{MarkupHandler, RegexMatch, TagLookup, TEXT_ANALYZER},
};
use gtk::{glib, pango, prelude::*, subclass::prelude::*};

const G_LOG_DOMAIN: &str = "ManuscriptBuffer";

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ManuscriptBuffer {}

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptBuffer {
        const NAME: &'static str = "ManuscriptBuffer";
        type Type = super::ManuscriptBuffer;
        type ParentType = gtk::TextBuffer;
    }

    impl ObjectImpl for ManuscriptBuffer {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().bind_default_tags();
        }
    }
    impl TextBufferImpl for ManuscriptBuffer {}
}

glib::wrapper! {
    pub struct ManuscriptBuffer(ObjectSubclass<imp::ManuscriptBuffer>) @extends gtk::TextBuffer;
}

impl Default for ManuscriptBuffer {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ManuscriptBuffer {
    pub fn new(tag_table: Option<gtk::TextTagTable>) -> Self {
        glib::Object::new(&[("tag-table", &tag_table)])
    }

    pub fn format_for(&self, view: &gtk::TextView) {
        let tags = TEXT_ANALYZER.analyze_buffer(self.upcast_ref::<gtk::TextBuffer>(), view);
        self.apply(tags, view);
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
                    ("pixels-above-lines", &0),
                    ("pixels-below-lines", &0),
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
