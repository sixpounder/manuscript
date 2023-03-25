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
            let (margin, indent) = get_margin_indent(self, margin_level, indent_level, None, None);
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
    markup_regex: RegexRuleCollection,
}

impl TextAnalyzer {
    pub fn new() -> Self {
        Self {
            markup_regex: RegexRuleCollection::new(),
        }
    }

    pub fn analyze_buffer(
        &self,
        buffer: &gtk::TextBuffer,
        view: &gtk::TextView,
    ) -> Vec<TagApplyRules> {
        let text = bytes_from_text_buffer(buffer).to_vec();
        let text: &str = std::str::from_utf8(&text).unwrap();
        let mut results = vec![];
        let mut match_count = 0;
        for rule in self.markup_regex.rules() {
            // let tag_rule = rule.regex();
            let tag_name = rule.name();
            let re = rule.regex();
            let matches = re.find_iter(text);
            for matched in matches {
                match_count += 1;
                let new_values = rule.map(&RegexMatch::new(re, matched, tag_name), view);
                results.push(new_values);
            }
        }
        glib::g_debug!(
            G_LOG_DOMAIN,
            "{} matches by regex -> {} rules emitted by text analysis",
            match_count,
            results.len()
        );

        results
    }
}

#[derive(Clone, Debug)]
pub struct RegexMatch<'a> {
    regex: &'a Regex,
    re_match: regex::Match<'a>,
    tag_name: &'a str,
}

impl<'a> RegexMatch<'a> {
    pub fn new(regex: &'a Regex, re_match: regex::Match<'a>, tag_name: &'a str) -> Self {
        Self {
            regex,
            re_match,
            tag_name,
        }
    }

    pub fn regex(&self) -> &'a Regex {
        self.regex
    }

    pub fn matched(&self) -> &regex::Match {
        &self.re_match
    }

    #[allow(dead_code)]
    pub fn tag_name(&self) -> &str {
        self.tag_name
    }

    pub fn start(&self) -> i32 {
        self.matched().start().try_into().unwrap()
    }

    pub fn end(&self) -> i32 {
        self.matched().end().try_into().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct TagApplyRules {
    header_candidate: Option<String>,
    lookups: Vec<TagLookup>,
}

impl TagApplyRules {
    pub fn new(lookups: Vec<TagLookup>) -> Self {
        Self {
            header_candidate: None,
            lookups,
        }
    }

    pub fn rules(&self) -> &Vec<TagLookup> {
        self.lookups.as_ref()
    }

    pub fn rules_mut(&mut self) -> &mut Vec<TagLookup> {
        self.lookups.as_mut()
    }

    pub fn is_header(&self) -> bool {
        self.header_candidate.is_some()
    }

    pub fn header_candidate(&self) -> Option<&String> {
        self.header_candidate.as_ref()
    }

    pub fn set_header_candidate(&mut self, value: Option<String>) {
        self.header_candidate = value;
    }
}

#[derive(Debug, Clone)]
pub enum TagLookup {
    ByName(&'static str, i32, i32),
    ByValue(gtk::TextTag, i32, i32),
}

#[derive(Debug)]
struct RegexRuleCollection {
    rules: Vec<RegexRule>,
}

impl Default for RegexRuleCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl RegexRuleCollection {
    fn create_regex<F>(collection: &mut Vec<RegexRule>, name: &str, re_content: &str, map: F)
    where
        F: Fn(&RegexMatch, &gtk::TextView) -> TagApplyRules + 'static,
    {
        match Regex::new(re_content) {
            Ok(regex) => {
                collection.push(RegexRule::new(String::from(name), regex, map));
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
            "HEADER",
            r"(?m)^ {0,3}(?P<level>#{1,6}) (?P<text>[^\n]+)$",
            |matched: &RegexMatch, view: &gtk::TextView| {
                let capture = matched
                    .regex()
                    .captures(matched.matched().as_str())
                    .unwrap();
                let level = &capture["level"];
                let margin = -(level.len() as i32) - 1;
                let mut rules = TagApplyRules::new(vec![
                    TagLookup::ByValue(
                        view.margin_indent_tag(margin, 0),
                        matched.start(),
                        matched.end(),
                    ),
                    TagLookup::ByName(TAG_NAME_ACCENT, matched.start(), matched.end()),
                    TagLookup::ByName(TAG_NAME_BOLD, matched.start(), matched.end()),
                ]);

                if let Some(content) = &capture.get(2) {
                    rules.set_header_candidate(Some(content.as_str().into()));
                }

                rules
            },
        );

        Self::create_regex(
            &mut regexes,
            "BOLD",
            r"(\*\*|__)[^\s*](?P<text>.*?\S.*?)(\*\*|__)",
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_BOLD,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        Self::create_regex(
            &mut regexes,
            "ITALIC_ASTERISK",
            r"\*[^\s\*](?P<text>.*?\S?.*?)\*",
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_ITALIC,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        Self::create_regex(
            &mut regexes,
            "ITALIC_UNDERSCORE",
            r"_[^\s_](?P<text>.*?\S?.*?)_",
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_ITALIC,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        Self::create_regex(
            &mut regexes,
            "BOLD_ITALIC",
            r"(\*\*\*|___)[^\s*](?P<text>.*?\S.*?)(\*\*\*|___)",
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_BOLD_ITALIC,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        Self::create_regex(
            &mut regexes,
            "STRIKETHROUGH",
            r"~~(?P<text>.*?\S.*?)~~",
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_STRIKETHROUGH,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        Self::create_regex(
            &mut regexes,
            "LINK",
            r#"\[(?P<text>.*?)\]\((?P<url>.+?)(?: "(?P<title>.+)")?\)"#,
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_LINK_COLOR_TEXT,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        Self::create_regex(
            &mut regexes,
            "HEADER_UNDER",
            r"(?m)(?:^\n*|\n\n)(?P<text>[^\s].+)\n {0,3}[=\-]+(?: +?\n|$)",
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_BOLD,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        Self::create_regex(
            &mut regexes,
            "HORIZONTAL_RULE",
            r"(?:^|\n{2,})(?P<symbols> {0,3}[*\-_]{3,} *)(?:\n{2,}|$)",
            |matched: &RegexMatch, _view: &gtk::TextView| {
                let capture = matched
                    .regex()
                    .captures(matched.matched().as_str())
                    .unwrap();
                let _symbols = &capture["symbols"];

                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_CENTER,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        Self::create_regex(
            &mut regexes,
            "FOOTNOTE_ID",
            r"\[\^(?P<id>(?P<text>[^\s]+))\]",
            // r"[^\s]+\[\^(?P<id>(?P<text>[^\s]+))\]", <- This version to include preceding word
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_LINK_COLOR_TEXT,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        // This version of footnote only matches one liners. The commented version below matches
        // multilines, but it's currently limited by having a final newline since
        // the regex crate doesnt support look around. We can do better.
        Self::create_regex(
            &mut regexes,
            "FOOTNOTE",
            r"(?:^\n*|\n\n)\[\^(?P<id>[^\s]+)\]:\s?(?P<content>(?:[^\n]+)?)",
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_GRAY_TEXT,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        // Self::create_regex(
        //     &mut regexes,
        //     "FOOTNOTE",
        //     r"(?:^\n*|\n\n)\[\^(?P<id>[^\s]+)\]:\s?(?P<first_line>(?:[^\n]+)?)(?P<line>(?:\s{4,}[^\n]+|\n+)+)",
        //     |matched: &RegexMatch, _view: &gtk::TextView| {
        //         vec![TagLookup::ByName(
        //             TAG_NAME_GRAY_TEXT,
        //             matched.start(),
        //             matched.end(),
        //         )]
        //     },
        // );

        Self::create_regex(
            &mut regexes,
            "SUBSCRIPT",
            r"~[^\n*].*~",
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_SUBSCRIPT,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        Self::create_regex(
            &mut regexes,
            "SUPERSCRIPT",
            r"\^[^\n*].*\^",
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_SUPERSCRIPT,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        Self::create_regex(
            &mut regexes,
            "CODE_BLOCK",
            r"(?ms)^ {0,3}(?P<block>([`~]{3})(?P<text>.+?)`{3})(?:\s+?$|$)",
            |matched: &RegexMatch, view: &gtk::TextView| {
                TagApplyRules::new(vec![
                    TagLookup::ByName(TAG_NAME_CODE_BLOCK, matched.start(), matched.end()),
                    TagLookup::ByValue(
                        view.margin_indent_tag(0, 1),
                        matched.start(),
                        matched.end(),
                    ),
                ])
            },
        );

        Self::create_regex(
            &mut regexes,
            "CODE",
            r"(?P<ticks_start>`+)(?P<content>.+?)(?P<ticks_end>`+)",
            |matched: &RegexMatch, _view: &gtk::TextView| {
                TagApplyRules::new(vec![TagLookup::ByName(
                    TAG_NAME_CODE_TEXT,
                    matched.start(),
                    matched.end(),
                )])
            },
        );

        // Self::create_regex(
        //     &mut regexes,
        //     "LINK_ALT",
        //     r"<(?P<text>(?P<url>[A-Za-z][A-Za-z0-9.+-]{1,31}:[^<>\x00-\x20]*|(?:[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*)))>",
        // );
        // Self::create_regex(
        //     &mut regexes,
        //     "IMAGE",
        //     r#"!\[(?P<text>.*)\]\((?P<url>.+?)(?: "(?P<title>.+)")?\)"#,
        // );

        // Self::create_regex(
        //     &mut regexes,
        //     "LIST",
        //     r"(?:^|\n)(?P<content>(?P<indent>(?:\t| {4})*)(?P<symbol>(?:[\-*+])) (?:\t| {4})*(?P<text>.+(?:\n+)*)?)",
        // );
        // Self::create_regex(
        //     &mut regexes,
        //     "ORDERED_LIST",
        //     r"(?:^|\n)(?P<content>(?P<indent>(?:\t| {4})*)(?P<prefix>(?:(?P<number>\d)|(?:[a-z]))+(?P<delimiter>[.)]))(?:\t| {4}| )(?P<text>.+(?:\n+)*)?)",
        // );

        // Self::create_regex(&mut regexes, "BLOCK_QUOTE", r"^ {0,3}(?:> ?)+(?P<text>.+)");

        // Self::create_regex(
        //     &mut regexes,
        //     "TABLE",
        //     r"^[\-+]{5,}\n(?P<text>.+?)\n[\-+]{5,}\n",
        // );

        // Self::create_regex(&mut regexes, "MENTION", r"@(?P<content>.+?)@");

        // regexes.push(MatchableTag::new(
        //     "MATH",
        //     Regex::new(r"([$]{1,2})(?P<text>[^`\\ ]{1,2}|[^` ].+?[^`\\ ])\1").unwrap(),
        // ));

        Self { rules: regexes }
    }

    pub fn rules(&self) -> &Vec<RegexRule> {
        self.rules.as_ref()
    }
}

type RegexRuleMapFn = Box<dyn Fn(&RegexMatch, &gtk::TextView) -> TagApplyRules + 'static>;

struct RegexRule {
    name: String,
    regex: Regex,
    map_fn: RegexRuleMapFn,
}

impl std::fmt::Debug for RegexRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}, regex: {}", self.name(), self.regex())
    }
}

unsafe impl Send for RegexRule {}
unsafe impl Sync for RegexRule {}

impl RegexRule {
    pub fn new<F>(name: String, regex: Regex, map: F) -> Self
    where
        F: Fn(&RegexMatch, &gtk::TextView) -> TagApplyRules + 'static,
    {
        Self {
            name,
            regex,
            map_fn: Box::new(map),
        }
    }

    #[allow(dead_code)]
    pub fn new_from_slice<F>(name: &'static str, regex: Regex, map: F) -> Self
    where
        F: Fn(&RegexMatch, &gtk::TextView) -> TagApplyRules + 'static,
    {
        Self::new(name.into(), regex, map)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn regex(&self) -> &Regex {
        &self.regex
    }

    pub fn map(&self, matched: &RegexMatch, view: &gtk::TextView) -> TagApplyRules {
        (self.map_fn)(matched, view)
    }
}
