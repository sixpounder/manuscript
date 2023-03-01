use gettextrs::{gettext, ngettext};

pub fn i18n(format: &str) -> String {
    gettext(format)
}

#[allow(dead_code)]
pub fn ni18n(single: &str, multiple: &str, number: u32) -> String {
    ngettext(single, multiple, number)
}

pub fn translators_list() -> Vec<&'static str> {
    vec!["Andrea Coronese (English, Italian)"]
}
