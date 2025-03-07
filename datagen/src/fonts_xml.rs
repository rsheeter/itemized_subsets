//! Mirrors Android's fonts.xml structure

use crate::fonts_xml_reader;

#[derive(Debug, Clone)]
pub struct Familyset(pub(crate) Vec<Entry>);

#[derive(Debug, Clone)]
pub(crate) enum Entry {
    Alias(Alias),
    Family(Family),
}

#[derive(Debug, Clone)]
pub struct Alias {
    pub name: Option<String>,
    pub to: Option<String>,
    pub weight: Option<i32>,
}

/// <https://developer.android.com/ndk/reference/group/font>
#[derive(Default, Debug, Clone)]
pub enum Style {
    #[default]
    Normal,
    Italic,
}

/// <https://developer.android.com/ndk/reference/group/font>
#[derive(Default, Debug, Clone)]
pub enum Variant {
    #[default]
    Default,
    Compact,
    Elegant,
}

#[derive(Debug, Clone)]
pub struct Family {
    pub name: Option<String>,
    pub variant: Variant,
    pub lang: Option<String>,
    pub fonts: Vec<Font>,
}

#[derive(Debug, Clone)]
pub struct Font {
    pub weight: i32,
    pub index: Option<i32>,
    pub filename: String,
    pub style: Style,
    pub fallback_for: Option<String>,
    pub post_script_name: Option<String>,
}

impl Familyset {
    pub fn fallbacks(&self) -> Vec<&Family> {
        self.0
            .iter()
            .filter_map(|e| match e {
                Entry::Family(f) => Some(f),
                Entry::Alias(_) => None,
            })
            .collect()
    }
}

impl Familyset {
    pub fn bundled_fonts_xml() -> Self {
        fonts_xml_reader::bundled_fonts_xml()
    }
}
