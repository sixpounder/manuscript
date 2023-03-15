use gtk::prelude::{TextViewExt, WidgetExt};

use std::cmp::max;

/// Gets the margin indent couple for given levels
pub fn get_margin_indent(
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
pub fn get_char_width(text_view: &gtk::TextView) -> i32 {
    gtk::pango::units_to_double(
        text_view
            .pango_context()
            .metrics(None, None)
            .approximate_char_width(),
    ) as i32
}
