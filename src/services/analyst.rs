use super::prelude::*;
use crate::libs::{consts::*, text_metrics::*};
use gtk::prelude::{TextBufferExt, TextBufferExtManual, TextViewExt};
use lazy_static::lazy_static;
use regex::Regex;

const G_LOG_DOMAIN: &str = "ManuscriptAnalyst";

pub trait MarkupHandler {
    fn tag(&self, name: &str) -> Option<gtk::TextTag>;
    fn margin_indent_tag(&self, margin_level: i32, indent_level: i32) -> gtk::TextTag;
}

impl MarkupHandler for gtk::TextView {
    fn tag(&self, name: &str) -> Option<gtk::TextTag> {
        let tag_table = self.buffer().tag_table();
        tag_table.lookup(name)
    }

    fn margin_indent_tag(&self, margin_level: i32, indent_level: i32) -> gtk::TextTag {
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

    pub fn analyze_buffer(&self, buffer: &gtk::TextBuffer, _view: &gtk::TextView) -> Vec<TagMatch> {
        let text = bytes_from_text_buffer(buffer).to_vec();
        let text: &str = std::str::from_utf8(&text).unwrap();
        let mut results = vec![];
        let mut match_count = 0;
        for rule in self.markup_regex.rules() {
            let tag_rule = rule.regex();
            let tag_name = rule.name();
            let re = rule.regex();
            let matches = re.find_iter(text);
            for matched in matches {
                match_count += 1;
                match tag_name {
                    "HEADER" => {
                        let capture = re.captures(matched.as_str()).unwrap();
                        let level = &capture["level"];
                        let margin = (level.len() as i32 * -1) - 1;
                        results.push(TagMatch::new(
                            tag_rule.as_str(),
                            tag_name,
                            matched.start(),
                            matched.end(),
                            TAG_NAME_MARGIN_INDENT,
                            Some((margin, 0)),
                        ));
                        results.push(TagMatch::new(
                            tag_rule.as_str(),
                            tag_name,
                            matched.start(),
                            matched.end(),
                            TAG_NAME_BOLD,
                            None,
                        ));
                    }
                    _ => (),
                }
            }
        }
        glib::g_debug!(
            G_LOG_DOMAIN,
            "{} matches by regex -> {} tags emitted by text analysis",
            match_count,
            results.len()
        );
        results
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct TagMatch<'a> {
    tag_rule: &'a str,
    tag_name: &'a str,
    start: usize,
    end: usize,
    target_tag_name: &'a str,
    args: Option<(i32, i32)>,
}

#[allow(dead_code)]
impl<'a> TagMatch<'a> {
    pub fn new(
        tag_rule: &'a str,
        tag_name: &'a str,
        start: usize,
        end: usize,
        target_tag_name: &'a str,
        args: Option<(i32, i32)>,
    ) -> Self {
        Self {
            tag_rule,
            tag_name,
            start,
            end,
            target_tag_name,
            args,
        }
    }

    pub fn tag_rule(&self) -> &str {
        self.tag_rule
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

    pub fn target_tag_name(&self) -> &str {
        self.target_tag_name
    }

    pub fn args(&self) -> Option<(i32, i32)> {
        self.args
    }
}

#[derive(Debug, Clone)]
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
            r"(?m)^ {0,3}(?P<level>#{1,6}) (?P<text>[^\n]+)$",
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

#[derive(Debug, Clone)]
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
