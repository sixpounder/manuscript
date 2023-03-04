use crate::{
    models::{ChunkType, DocumentChunk},
    services::i18n::i18n,
    widgets::ManuscriptChunkRow,
};
use adw::prelude::ActionRowExt;
use gtk::prelude::*;

/// Checks if `parent` contains an expander row for `chunk`'s category and, if not,
/// adds it to `parent`. Returns the created or found `adw::ExpanderRow`
pub fn get_or_create_expander_row_for_chunk(
    parent: &gtk::Widget,
    chunk: &dyn DocumentChunk,
) -> adw::ExpanderRow {
    let mut child = parent.first_child();
    let mut existing_expander = None;
    while child.is_some() {
        let existing_child = child.unwrap();
        child = existing_child.next_sibling();
        let maybe_data = unsafe { existing_child.data::<ChunkType>("chunk_type") };
        if let Some(inner_data) = maybe_data {
            let inner_data = unsafe { inner_data.as_ref() };
            if *inner_data == chunk.chunk_type() {
                existing_expander = Some(
                    existing_child
                        .downcast::<adw::ExpanderRow>()
                        .expect("Not an adw::ExpanderRow"),
                );
            }
        }
    }

    existing_expander.unwrap_or_else(|| {
        let expander_row = create_expander_row_for_chunk(chunk);
        if let Some(parent_box) = parent.downcast_ref::<gtk::Box>() {
            parent_box.append(&expander_row);
        } else {
            expander_row.set_parent(parent);
        }
        expander_row
    })
}

pub fn create_expander_row_for_chunk(chunk: &dyn DocumentChunk) -> adw::ExpanderRow {
    let expander_row = adw::ExpanderRow::builder()
        .hexpand(true)
        .halign(gtk::Align::Fill)
        .title(chunk.category_name().as_str())
        .build();
    unsafe {
        expander_row.set_data("chunk_type", chunk.chunk_type());
    };
    expander_row
}

/// Finds the `adw::ActionRow` related to `chunk` inside `parent`. If not found, creates
/// it and adds it to the proper category expander.
pub fn get_or_create_row_for_chunk(
    parent: &gtk::Widget,
    chunk: &dyn DocumentChunk,
) -> ManuscriptChunkRow {
    let mut child = parent.first_child();
    let mut existing_row = None;
    while child.is_some() {
        let existing_child = child.unwrap();
        child = existing_child.next_sibling();
        let maybe_data = unsafe { existing_child.data::<String>("chunk_id") };
        if let Some(inner_data) = maybe_data {
            let inner_data = unsafe { inner_data.as_ref() };
            if *inner_data == chunk.id().to_string() {
                existing_row = Some(
                    existing_child
                        .downcast::<ManuscriptChunkRow>()
                        .expect("Not a ManuscriptChunkRow"),
                );
            }
        }
    }

    existing_row.unwrap_or_else(|| create_row_for_chunk(chunk))
}

pub fn create_row_for_chunk(chunk: &dyn DocumentChunk) -> ManuscriptChunkRow {
    ManuscriptChunkRow::new(Some(chunk))
}
