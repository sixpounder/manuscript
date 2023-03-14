use super::prelude::*;
use gtk::{
    pango,
    prelude::{TextBufferExt, TextBufferExtManual, TextViewExt, WidgetExt},
};
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::max;

const G_LOG_DOMAIN: &str = "ManuscriptAnalyst";

const TAG_NAME_ITALIC: &str = "italic";
const TAG_NAME_BOLD: &str = "bold";
const TAG_NAME_BOLD_ITALIC: &str = "bold_italic";
const TAG_NAME_STRIKETHROUGH: &str = "strikethrough";
const TAG_NAME_CENTER: &str = "center";
const TAG_NAME_WRAP_NONE: &str = "wrap_none";
const TAG_NAME_PLAIN_TEXT: &str = "plain_text";
const TAG_NAME_GRAY_TEXT: &str = "gray_text";
const TAG_NAME_LINK_COLOR_TEXT: &str = "link_color_text";
const TAG_NAME_CODE_TEXT: &str = "code_text";
const TAG_NAME_CODE_BLOCK: &str = "code_block";
const TAG_NAME_UNFOCUSED_TEXT: &str = "unfocused_text";
// const TAG_NAME_MARGIN_INDENT: &str = "margin_indent";

const PANGO_WEIGHT_NORMAL: i32 = 400;
const PANGO_WEIGHT_BOLD: i32 = 700;

pub trait MarkupHandler {
    fn bind_default_tags(&self);
    fn tag(&self, name: &str) -> Option<gtk::TextTag>;
    fn margin_indent_tag(&mut self, margin_level: i32, indent_level: i32) -> gtk::TextTag;
}

impl MarkupHandler for gtk::TextView {
    fn bind_default_tags(&self) {
        let buffer = self.buffer();

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
            .create_tag(Some(TAG_NAME_STRIKETHROUGH), &[("striketrough", &true)])
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
                    ("pixels-above-line", &0),
                    ("pixels-below-line", &0),
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

        buffer
            .create_tag(
                Some(TAG_NAME_CODE_BLOCK),
                &[
                    ("weight", &PANGO_WEIGHT_NORMAL),
                    ("style", &pango::Style::Normal),
                    ("strikethrough", &false),
                    ("indent", &get_margin_indent(self, 0, 1, None, None).1),
                ],
            )
            .unwrap();
    }

    fn tag(&self, name: &str) -> Option<gtk::TextTag> {
        let tag_table = self.buffer().tag_table();
        tag_table.lookup(name)
    }

    fn margin_indent_tag(&mut self, margin_level: i32, indent_level: i32) -> gtk::TextTag {
        let tag_name = format!("margin_indent_{margin_level}_{indent_level}");
        if let Some(tag) = self.tag(tag_name.as_str()) {
            tag
        } else {
            let (margin, indent) = get_margin_indent(&self, margin_level, indent_level, None, None);
            self.buffer()
                .create_tag(
                    Some(tag_name.as_str()),
                    &[("left-margin", &margin), ("indent", &indent)],
                )
                .unwrap();
            self.tag(tag_name.as_str()).unwrap()
        }
    }
}

/// Gets the margin indent couple for given levels
fn get_margin_indent(
    text_view: &gtk::TextView,
    margin_level: i32,
    indent_level: i32,
    baseline_margin: Option<i32>,
    char_width: Option<i32>,
) -> (i32, i32) {
    let baseline_margin = baseline_margin.unwrap_or_else(|| text_view.left_margin());
    let char_width = char_width.unwrap_or_else(|| get_char_width(text_view));

    let margin = max(baseline_margin + char_width * margin_level, 0);
    let indent = char_width * indent_level;

    (margin, indent)
}

/// Estimate the char width on a text view
fn get_char_width(text_view: &gtk::TextView) -> i32 {
    gtk::pango::units_to_double(
        text_view
            .pango_context()
            .metrics(None, None)
            .approximate_char_width(),
    ) as i32
}

lazy_static! {
    pub static ref TEXT_ANALYZER: TextAnalyzer = TextAnalyzer::new();
}

pub struct TextAnalyzer {
    markup_regex: MarkupRegex,
}

impl TextAnalyzer {
    pub fn new() -> Self {
        Self {
            markup_regex: MarkupRegex::new(),
        }
    }

    pub fn analyze_buffer(&self, buffer: &gtk::TextBuffer) -> Vec<TagMatch> {
        let text = bytes_from_text_buffer(buffer).to_vec();
        let text: &str = std::str::from_utf8(&text).unwrap();
        self.analyze(text)
    }

    pub fn analyze(&self, text: &str) -> Vec<TagMatch> {
        let mut results = vec![];
        for rule in self.markup_regex.rules() {
            let tag_name = rule.name();
            let re = rule.regex();
            let matches = re.find_iter(text);
            for matched in matches {
                results.push(TagMatch::new(tag_name, matched.start(), matched.end()));
            }
        }
        glib::g_debug!(G_LOG_DOMAIN, "Found {} matching tag rules", results.len());
        results
    }
}

pub struct TagMatch<'a> {
    tag_name: &'a str,
    start: usize,
    end: usize,
}

impl<'a> TagMatch<'a> {
    pub fn new(tag_name: &'a str, start: usize, end: usize) -> Self {
        Self {
            tag_name,
            start,
            end,
        }
    }

    pub fn tag_name(&self) -> &str {
        self.tag_name
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

#[derive(Clone)]
pub struct TagRule {
    name: String,
    regex: Regex,
}

impl TagRule {
    pub fn new(name: String, regex: Regex) -> Self {
        Self { name, regex }
    }

    pub fn new_from_slice(name: &'static str, regex: Regex) -> Self {
        Self {
            name: String::from(name),
            regex,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn regex(&self) -> &Regex {
        &self.regex
    }
}

pub struct MarkupRegex {
    rules: Vec<TagRule>,
}

impl Default for MarkupRegex {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkupRegex {
    fn create_regex(collection: &mut Vec<TagRule>, name: &str, re_content: &str) {
        match Regex::new(re_content) {
            Ok(regex) => {
                collection.push(TagRule::new(String::from(name), regex));
            }
            Err(err) => {
                glib::g_warning!(
                    G_LOG_DOMAIN,
                    "Unable to create regex from {}: {}",
                    re_content,
                    err
                );
            }
        }
    }

    pub fn new() -> Self {
        let mut regexes = Vec::with_capacity(10);

        Self::create_regex(
            &mut regexes,
            "ITALIC_ASTERISK",
            r"\*[^\s\*](?P<text>.*?\S?.*?)\*",
        );
        Self::create_regex(
            &mut regexes,
            "ITALIC_UNDERSCORE",
            r"_[^\s_](?P<text>.*?\S?.*?)_",
        );
        Self::create_regex(
            &mut regexes,
            "BOLD_ITALIC",
            r"(\*\*|__)[^\s*](?P<text>.*?\S.*?)(\*\*|__)",
        );
        Self::create_regex(&mut regexes, "STRIKETHROUGH", r"~~(?P<text>.*?\S.*?)~~");
        Self::create_regex(
            &mut regexes,
            "CODE",
            r"(?P<ticks_start>`+)(?P<content>.+?)(?P<ticks_end>`+)",
        );
        Self::create_regex(
            &mut regexes,
            "LINK",
            r#"\[(?P<text>.*?)\]\((?P<url>.+?)(?: "(?P<title>.+)")?\)"#,
        );
        Self::create_regex(
            &mut regexes,
            "LINK_ALT",
            r"<(?P<text>(?P<url>[A-Za-z][A-Za-z0-9.+-]{1,31}:[^<>\x00-\x20]*|(?:[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*)))>",
        );
        Self::create_regex(
            &mut regexes,
            "IMAGE",
            r#"!\[(?P<text>.*)\]\((?P<url>.+?)(?: "(?P<title>.+)")?\)"#,
        );
        Self::create_regex(
            &mut regexes,
            "HORIZONTAL_RULE",
            r"(?:^|\n{2,})(?P<symbols> {0,3}[*\-_]{3,} *)(?:\n{2,}|$)",
        );
        Self::create_regex(
            &mut regexes,
            "LIST",
            r"(?:^|\n)(?P<content>(?P<indent>(?:\t| {4})*)(?P<symbol>(?:[\-*+])) (?:\t| {4})*(?P<text>.+(?:\n+)*)?)",
        );
        Self::create_regex(
            &mut regexes,
            "ORDERED_LIST",
            r"(?:^|\n)(?P<content>(?P<indent>(?:\t| {4})*)(?P<prefix>(?:(?P<number>\d)|(?:[a-z]))+(?P<delimiter>[.)]))(?:\t| {4}| )(?P<text>.+(?:\n+)*)?)",
        );
        Self::create_regex(&mut regexes, "BLOCK_QUOTE", r"^ {0,3}(?:> ?)+(?P<text>.+)");
        Self::create_regex(
            &mut regexes,
            "HEADER",
            r"^ {0,3}(?P<level>#{1,6}) (?P<text>[^\n]+)",
        );
        Self::create_regex(
            &mut regexes,
            "HEADER_UNDER",
            r"(?:^\n*|\n\n)(?P<text>[^\s].+)\n {0,3}[=\-]+(?: +?\n|$)",
        );
        Self::create_regex(
            &mut regexes,
            "TABLE",
            r"^[\-+]{5,}\n(?P<text>.+?)\n[\-+]{5,}\n",
        );
        Self::create_regex(
            &mut regexes,
            "FOOTNOTE_ID",
            r"[^\s]+\[\^(?P<id>(?P<text>[^\s]+))\]",
        );
        Self::create_regex(
            &mut regexes,
            "FOOTNOTE",
            r"(?:^\n*|\n\n)\[\^(?P<id>[^\s]+)\]:\s?(?P<first_line>(?:[^\n]+)?)(?P<line>(?:\s{4,}[^\n]+|\n+)+)",
        );
        Self::create_regex(&mut regexes, "MENTION", r"@(?P<content>.+?)@");

        // regexes.push(MarkupMatch::new(
        //     "CODE_BLOCK",
        //     Regex::new(r"^ {0,3}(?P<block>([`~]{3})(?P<text>.+?)(?<! ) {0,3}\2)(?:\s+?$|$)")
        //         .unwrap(),
        // ));

        // regexes.push(MatchableTag::new(
        //     "MATH",
        //     Regex::new(r"([$]{1,2})(?P<text>[^`\\ ]{1,2}|[^` ].+?[^`\\ ])\1").unwrap(),
        // ));

        Self { rules: regexes }
    }

    pub fn rules(&self) -> &Vec<TagRule> {
        self.rules.as_ref()
    }
}
