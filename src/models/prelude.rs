use super::{Chapter, CharacterSheet};
use bytes::Bytes;
use glib::{StaticType, Type};
use gtk::gdk::RGBA;
use serde::{
    de::{Deserializer, Error, Visitor},
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};
use std::ops::{Deref, DerefMut};

fn default_contrast_light() -> Color {
    Color::new(0.0, 0.0, 0.0, 0.9)
}

fn default_contrast_dark() -> Color {
    Color::new(250.0, 250.0, 250.0, 0.9)
}

/// Wraps a `gdk::RGBA` adding more capabilities like dark/light
/// detection and (de)serialization support
#[derive(glib::ValueDelegate, Debug, Clone, Copy)]
#[value_delegate(nullable)]
pub struct Color(RGBA);

impl Color {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self(RGBA::new(red, green, blue, alpha))
    }

    pub fn is_dark(&self) -> bool {
        !self.is_light()
    }

    // Calculate the perceived brightness of the color using the formula
    // (0.299 * R + 0.587 * G + 0.114 * B) * A
    pub fn is_light(&self) -> bool {
        let brightness = (0.299 * self.0.red() + 0.587 * self.0.green() + 0.114 * self.0.blue())
            * self.0.alpha();
        // If the brightness is greater than or equal to 0.6, the color is considered light
        brightness >= 0.6
    }

    pub fn contrast_color(&self) -> Color {
        if self.is_light() {
            default_contrast_light()
        } else {
            default_contrast_dark()
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::from(RGBA::TRANSPARENT)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rgba({}, {}, {}, {})",
            self.0.red() * 255.0,
            self.0.green() * 255.0,
            self.0.blue() * 255.0,
            self.0.alpha().clamp(0.0, 1.0)
        )
    }
}

impl From<gtk::gdk::RGBA> for Color {
    fn from(value: gtk::gdk::RGBA) -> Self {
        Self(value)
    }
}

impl From<Color> for gtk::gdk::RGBA {
    fn from(value: Color) -> Self {
        value.0
    }
}

impl From<&Color> for gtk::gdk::RGBA {
    fn from(value: &Color) -> Self {
        value.0
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 4 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Color", 4)?;
        state.serialize_field("r", &self.0.red())?;
        state.serialize_field("g", &self.0.green())?;
        state.serialize_field("b", &self.0.blue())?;
        state.serialize_field("a", &self.0.alpha())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Red,
            Green,
            Blue,
            Alpha,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("Color field")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            "red" => Ok(Field::Red),
                            "green" => Ok(Field::Green),
                            "blue" => Ok(Field::Blue),
                            "alpha" => Ok(Field::Alpha),
                            _ => Err(Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Color")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let red = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;
                let green = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;
                let blue = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;
                let alpha = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;
                Ok(Color::new(red, green, blue, alpha))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut red = None;
                let mut green = None;
                let mut blue = None;
                let mut alpha = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Red => {
                            if red.is_some() {
                                return Err(Error::duplicate_field("red"));
                            }
                            red = Some(map.next_value()?);
                        }
                        Field::Green => {
                            if green.is_some() {
                                return Err(Error::duplicate_field("green"));
                            }
                            green = Some(map.next_value()?);
                        }
                        Field::Blue => {
                            if blue.is_some() {
                                return Err(Error::duplicate_field("blue"));
                            }
                            blue = Some(map.next_value()?);
                        }
                        Field::Alpha => {
                            if alpha.is_some() {
                                return Err(Error::duplicate_field("alpha"));
                            }
                            alpha = Some(map.next_value()?);
                        }
                    }
                }
                let red = red.ok_or_else(|| Error::missing_field("red"))?;
                let green = green.ok_or_else(|| Error::missing_field("green"))?;
                let blue = blue.ok_or_else(|| Error::missing_field("blue"))?;
                let alpha = alpha.ok_or_else(|| Error::missing_field("alpha"))?;

                Ok(Color::new(red, green, blue, alpha))
            }
        }

        const FIELDS: &[&str] = &["red", "green", "blue", "alpha"];
        deserializer.deserialize_struct("Duration", FIELDS, ColorVisitor)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Manifest,
    Chapter,
    CharacterSheet,
}

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = match self {
            ChunkType::Manifest => "Manifest",
            ChunkType::Chapter => "Chapter",
            ChunkType::CharacterSheet => "Character Sheet",
        };
        write!(f, "{desc}")
    }
}

#[derive(Debug, Clone)]
pub enum ManuscriptError {
    NoDocument,
    NoBackend,
    Save,
    Open(String),
    DocumentLock,
    DocumentSerialize,
    DocumentDeserialize,
    ChunkParse,
    ChunkBusy,
    ChunkUnavailable,
    Reason(&'static str),
}

pub type ManuscriptResult<T> = Result<T, ManuscriptError>;

pub trait DocumentChunk {
    fn id(&self) -> &str;
    fn title(&self) -> Option<String>;
    fn default_title(&self) -> String;
    fn chunk_type(&self) -> ChunkType;
    fn category_name(&self) -> String;
    fn priority(&self) -> Option<u64>;
    fn set_priority(&mut self, value: Option<u64>);
    fn locked(&self) -> bool;
    fn set_locked(&mut self, value: bool);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn accent(&self) -> Option<Color> {
        None
    }

    fn set_accent(&mut self, _value: Option<Color>) -> ManuscriptResult<()> {
        Err(ManuscriptError::Reason("Not implemented"))
    }

    fn safe_title(&self) -> String {
        self.title().unwrap_or(self.default_title())
    }

    fn heading(&self) -> String {
        self.safe_title()
    }

    fn include_in_compilation(&self) -> bool {
        true
    }

    fn set_include_in_compilation(&mut self, _value: bool) -> ManuscriptResult<()> {
        Err(ManuscriptError::Reason("Not implemented"))
    }
}

impl StaticType for dyn DocumentChunk {
    fn static_type() -> Type {
        Type::OBJECT
    }
}

impl std::fmt::Debug for dyn DocumentChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.chunk_type(), self.id())
    }
}

impl dyn DocumentChunk {
    pub fn new_of_type<C>() -> C
    where
        C: DocumentChunk + Default,
    {
        C::default()
    }
}

impl<C> DocumentChunk for Box<C>
where
    C: DocumentChunk,
{
    fn id(&self) -> &str {
        self.deref().id()
    }

    fn title(&self) -> Option<String> {
        self.deref().title()
    }

    fn default_title(&self) -> String {
        self.deref().default_title()
    }

    fn heading(&self) -> String {
        self.deref().heading()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self.deref().as_any()
    }

    fn priority(&self) -> Option<u64> {
        self.deref().priority()
    }

    fn set_priority(&mut self, value: Option<u64>) {
        self.deref_mut().set_priority(value);
    }

    fn chunk_type(&self) -> ChunkType {
        self.deref().chunk_type()
    }

    fn category_name(&self) -> String {
        self.deref().category_name()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self.deref_mut().as_any_mut()
    }

    fn locked(&self) -> bool {
        self.deref().locked()
    }

    fn set_locked(&mut self, value: bool) {
        self.deref_mut().set_locked(value);
    }
}

impl Clone for Box<dyn DocumentChunk> {
    fn clone(&self) -> Self {
        let any = self.as_any();
        if let Some(chapter) = any.downcast_ref::<Chapter>() {
            Box::new(chapter.clone())
        } else if let Some(character_sheet) = any.downcast_ref::<CharacterSheet>() {
            Box::new(character_sheet.clone())
        } else {
            unreachable!()
        }
    }
}

pub trait SerializableChunk<'de>: DocumentChunk + Serialize + Deserialize<'de> {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagMark(i32, i32, String);

pub trait BufferChunk: DocumentChunk {
    fn buffer(&self) -> &Bytes;

    fn tags_map(&self) -> &Vec<TagMark> {
        static EMPTY_MARKS: once_cell::sync::Lazy<Vec<TagMark>> =
            once_cell::sync::Lazy::new(|| vec![]);
        EMPTY_MARKS.as_ref()
    }

    /// Counts the words in the buffer
    fn words_count(&self) -> u64 {
        self.buffer().words_count()
    }

    /// Estimates the reading time of this buffer
    fn estimate_reading_time(&self) -> (u64, u64) {
        self.buffer().estimate_reading_time()
    }
}

pub trait MutableBufferChunk: BufferChunk {
    fn set_buffer(&mut self, value: Bytes);
}

pub trait BufferAnalytics {
    /// Counts the words in the buffer
    fn words_count(&self) -> u64;

    /// Estimates the reading time of this buffer
    fn estimate_reading_time(&self) -> (u64, u64);
}

impl BufferAnalytics for Bytes {
    /// Counts the words in the buffer
    fn words_count(&self) -> u64 {
        let mut state = 0;
        let mut wc: u64 = 0;
        let buffer = self;
        for i in 0..buffer.len() {
            let c = buffer[i];
            if c == b' ' || c == b'\n' || c == b'\t' || c == b'\r' || c == b'#' {
                state = 0;
            } else if state == 0 {
                state = 1;
                wc += 1;
            }
        }

        wc
    }

    /// Estimates the reading time of this buffer
    fn estimate_reading_time(&self) -> (u64, u64) {
        let words = self.words_count();
        let words_divided_two_hundreds = words as f64 / 200.0;
        let minutes = words_divided_two_hundreds.floor() as u64;
        let seconds = (words_divided_two_hundreds * 0.60) as u64;

        (minutes, seconds)
    }
}
