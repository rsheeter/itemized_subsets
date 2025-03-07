//! Quick and dirty exposure of fonts.xml (from Android) to Rust

use std::str::from_utf8;

use quick_xml::{Reader, events::Event};

const FONTS_XML: &str = include_str!("../third_party/fonts.xml");

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

// Since these are always from FONTS_XML they should be able to be &'static str
fn optional_string(raw: &[u8]) -> Option<String> {
    if raw.is_empty() {
        return None;
    }
    Some(
        from_utf8(raw)
            .expect("Bundled fonts.xml is defective?!")
            .to_string(),
    )
}

fn optional_number(raw: &[u8]) -> Option<i32> {
    if raw.is_empty() {
        return None;
    }
    Some(
        from_utf8(raw)
            .expect("Bundled fonts.xml is defective?!")
            .parse()
            .expect("Number"),
    )
}

impl Familyset {
    pub fn from_fonts_xml() -> Self {
        let mut reader = Reader::from_str(FONTS_XML);
        reader.config_mut().trim_text(true);

        let mut num_familysets = 0;
        let mut entries = Vec::new();
        let mut font_in_progress = None;

        // We panic on unrecognized to enable testing to confirm we understood everything we saw
        loop {
            match reader.read_event() {
                Ok(Event::Decl(..)) | Ok(Event::Comment(..)) => (),
                Err(e) => panic!(
                    "Failed to read bundled fonts.xml at position {}: {:?}",
                    reader.error_position(),
                    e
                ),
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    match e.name().local_name().into_inner() {
                        b"familyset" => {
                            num_familysets += 1;
                            if num_familysets > 1 {
                                panic!("Found multiple familyset elements?!");
                            };
                        }
                        b"family" => {
                            let mut name = Default::default();
                            let mut lang = Default::default();
                            let mut variant = Variant::Default;
                            for attr in e.attributes() {
                                let Ok(attr) = attr else {
                                    panic!("Bundled fonts.xml is defective?! {attr:?}");
                                };
                                match attr.key.0 {
                                    b"name" => name = optional_string(&attr.value),
                                    b"lang" => lang = optional_string(&attr.value),
                                    b"variant" => {
                                        variant = match &*attr.value {
                                            b"compact" => Variant::Compact,
                                            b"elegant" => Variant::Elegant,
                                            v => panic!("Unknown variant {:?}", from_utf8(v)),
                                        }
                                    }
                                    _ => (),
                                }
                            }

                            entries.push(Entry::Family(Family {
                                name,
                                lang,
                                variant,
                                fonts: Vec::new(),
                            }));
                        }
                        b"font" => {
                            let mut weight = Default::default();
                            let mut index = Default::default();
                            let mut style = Default::default();
                            let mut fallback_for = Default::default();
                            let mut post_script_name = Default::default();
                            for attr in e.attributes() {
                                let Ok(attr) = attr else {
                                    panic!("Bundled fonts.xml is defective?! {attr:?}");
                                };
                                match attr.key.0 {
                                    b"weight" => weight = optional_number(&attr.value).unwrap(),
                                    b"index" => index = optional_number(&attr.value),
                                    b"style" => {
                                        style = match &*attr.value {
                                            b"normal" => Style::Normal,
                                            b"italic" => Style::Italic,
                                            v => panic!("Unknown style {:?}", from_utf8(v)),
                                        }
                                    }
                                    b"fallbackFor" => fallback_for = optional_string(&attr.value),
                                    b"postScriptName" => {
                                        post_script_name = optional_string(&attr.value)
                                    }
                                    v => panic!("Unknown font attribute {:?}", from_utf8(v)),
                                }
                            }
                            font_in_progress = Some(Font {
                                weight,
                                index,
                                filename: String::default(),
                                style,
                                fallback_for,
                                post_script_name,
                            })
                        }
                        b"alias" => {
                            let mut name = Default::default();
                            let mut to = Default::default();
                            let mut weight = Default::default();
                            for attr in e.attributes() {
                                let Ok(attr) = attr else {
                                    panic!("Bundled fonts.xml is defective?! {attr:?}");
                                };
                                match attr.key.0 {
                                    b"name" => name = optional_string(&attr.value),
                                    b"to" => to = optional_string(&attr.value),
                                    b"weight" => weight = optional_number(&attr.value),
                                    v => panic!("Unknown alias attribute {:?}", from_utf8(v)),
                                }
                            }
                            entries.push(Entry::Alias(Alias { name, to, weight }));
                        }
                        b"axis" => {
                            eprintln!("TODO axis");
                        }
                        v => panic!("Unsupported {:?}", from_utf8(v)),
                    }
                }
                Ok(Event::Text(e)) => {
                    if let Some(font) = font_in_progress.as_mut() {
                        font.filename += &*e.unescape().expect("Invalid bundled fonts.xml?!");
                    }
                }
                Ok(Event::End(e)) => {
                    if e.name().0 == b"font" {
                        if let Some(font) = font_in_progress.take() {
                            let Some(Entry::Family(family)) = entries.last_mut() else {
                                panic!("Bad bookkeeping");
                            };
                            family.fonts.push(font);
                        }
                    }
                }
                Ok(e) => panic!("Unsupported {:?}", e),
            }
        }

        println!("{} fonts.xml entries", entries.len());

        Familyset(entries)
    }

    pub fn fallbacks(&self) -> Vec<&Family> {
        self
        .0
        .iter()
        .filter_map(|e| match e {
            Entry::Family(f) => Some(f),
            Entry::Alias(_) => None,
        })
        .collect()
    }
}
