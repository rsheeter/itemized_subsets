//! Mirrors Android's fonts.xml structure

use regex::Regex;
use skrifa::Tag;

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
    pub weight: Option<f32>,
}

/// <https://developer.android.com/ndk/reference/group/font>
#[derive(Default, Debug, Clone, PartialEq)]
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
    pub ignore: bool,
    pub lang: Option<String>,
    pub fonts: Vec<Font>,
}

#[derive(Debug, Clone)]
pub struct Font {
    pub weight: f32,
    pub index: Option<i32>,
    pub filename: String,
    pub style: Style,
    pub fallback_for: Option<String>,
    pub post_script_name: Option<String>,
    pub location: Vec<AxisPosition>,
}

#[derive(Debug, Clone)]
pub struct AxisPosition {
    pub tag: Tag,
    pub value: f32,
}

impl Familyset {
    pub fn named(&self, name: &str) -> Option<&Family> {
        for e in self.0.iter() {
            let Entry::Family(family) = e else {
                continue;
            };
            if family.name.as_deref() == Some(name) {
                return Some(family);
            }
        }
        None
    }
    pub fn fallbacks(&self) -> Vec<&Family> {
        self.0
            .iter()
            .filter_map(|e| match e {
                Entry::Family(f) => (!f.ignore).then_some(f),
                Entry::Alias(_) => None,
            })
            .collect()
    }
}

impl Familyset {
    /// The contents of the bundled fonts.xml
    pub fn bundled_fonts_xml() -> Self {
        fonts_xml_reader::bundled_fonts_xml()
    }

    /// The contents of the bundled fonts.xml, adjusted to align with the Google Fonts web api
    ///
    /// For example, only one emoji file, CJK collections exploded into individual families
    pub fn fonts_xml_for_googlefonts() -> Self {
        let mut set = fonts_xml_reader::bundled_fonts_xml();

        // Android has 3 emoji families: COLR, CBDT, and flags. GF just has Noto Color Emoji.
        // Drop flags here. CBDT is dropped by the ignore check in fallbacks.
        set.0.retain(|e| match e {
            Entry::Family(f) => !f
                .fonts
                .iter()
                .all(|f| f.filename == "NotoColorEmojiFlags.ttf"),
            _ => true,
        });

        // The fallbacks for zh-Hant,zh-Bopo, ja, ko all use ttcs. Google Fonts uses single families.
        let re = Regex::new(r"^Noto(Sans|Serif|)CJK-Regular.ttc$").unwrap();
        for entry in set.0.iter_mut() {
            let Entry::Family(family) = entry else {
                continue;
            };
            if !family.fonts.iter().any(|f| f.filename.ends_with(".ttc")) {
                continue;
            }
            let suffix = match family.lang.as_deref() {
                Some("ja") => "JP",
                Some("ko") => "KR",
                Some("zh-Hant,zh-Bopo") => "TC",
                Some("zh-Hans") => "SC",
                _ => continue,
            };
            for font in family.fonts.iter_mut() {
                let Some(sans_or_serif) = re.captures(&font.filename) else {
                    panic!("Bad {font:?}");
                };
                let new_filename = format!(
                    "Noto{}{}.ttf",
                    sans_or_serif.get(1).unwrap().as_str(),
                    suffix
                );
                let new_postscript_name = format!(
                    "Noto {} {}.ttf",
                    sans_or_serif.get(1).unwrap().as_str(),
                    suffix
                );
                font.filename = new_filename;
                font.post_script_name = Some(new_postscript_name);
                font.index = None;
            }
        }

        set
    }
}
