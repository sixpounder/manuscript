use bytes::{BufMut, Bytes, BytesMut};
use gtk::prelude::{TextBufferExt, TextTagExt};

struct RichTextSerde;

impl RichTextSerde {
    /// A function that serializes a source `gtk::TextBuffer` into raw bytes while carrying informations
    /// about that buffer tags with it.
    fn serialize_buffer_with_tags(source: &gtk::TextBuffer) -> Bytes {
        let mut tags_stack = vec![];
        let mut bytes = BytesMut::new();
        let mut cursor = source.start_iter();
        let mut counter: u64 = 0;

        while !cursor.is_end() {
            if cursor.starts_tag(None::<&gtk::TextTag>) {
                let toggled_tags = cursor.toggled_tags(true);
                for tag in toggled_tags {
                    match tag.name() {
                        Some(name) => {
                            // Only consider non-anonymous tags...
                            match source.tag_table().lookup(name.as_str()) {
                                Some(existing_tag) => {
                                    // ... and only if they exist in the buffer tag table
                                    tags_stack.push(existing_tag);
                                    bytes.put_slice(format!("<{}>", name).as_bytes());
                                }
                                None => (),
                            }
                        }
                        None => (),
                    }
                }
            }

            check_closing_tags(counter, &mut bytes, &mut cursor, &mut tags_stack);
            put_char_into_buffer(cursor.char(), bytes);

            cursor.forward_char();
            counter += 1;
        }

        check_closing_tags(counter, &mut bytes, &mut cursor, &mut tags_stack);

        bytes.freeze()
    }

    fn deserialize_buffer_with_tags(source: Bytes) -> gtk::TextBuffer {
        todo!()
    }
}

fn check_closing_tags(
    counter: u64,
    bytes: &mut BytesMut,
    cursor: &mut gtk::TextIter,
    stack: &mut Vec<gtk::TextTag>,
) {
    if cursor.ends_tag(None::<&gtk::TextTag>) {
        let mut closed_tags = cursor.toggled_tags(false);
        while !closed_tags.is_empty() {
            // Get the current closing tag from the list, remove it from the list...
            closed_tags.pop();

            if !stack.is_empty() {
                // ... and pop it from the stack
                let tag_to_close = stack.pop().unwrap();
                bytes.put_slice(format!("</{}>", tag_to_close.name().unwrap()).as_bytes());
            } else {
                glib::g_warning!("check_closing_tags", "Be aware that the tag stack is null or zero sized when trying to pop an item. This is probably an error because this is called when an iter is closing some tag that should be present on the stack.");
            }
        }
    }
}

fn put_char_into_buffer(ch: char, buf: &mut BytesMut) {
    let mut b: Vec<u8> = Vec::with_capacity(ch.len_utf8());
    ch.encode_utf8(&mut b);
    buf.put_slice(b.as_slice());
}
