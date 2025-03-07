//! Exploratory hackery
use std::collections::HashMap;

use fonts::Fonts;
use fonts_xml::Entry;
use itertools::Itertools;

use icu_segmenter::GraphemeClusterSegmenter;

pub mod fonts;
pub(crate) mod fonts_xml;

pub fn graphemes(s: &str) -> Vec<&str> {
    GraphemeClusterSegmenter::new()
        .segment_str(s)
        .tuple_windows()
        .map(|(i, j)| &s[i..j])
        .collect()
}

pub fn itemize(s: &str, fonts: Fonts) {
    let familyset = fonts_xml::Familyset::from_fonts_xml();
    let fallbacks = familyset
        .0
        .iter()
        .filter_map(|e| match e {
            Entry::Family(f) => Some(f),
            Entry::Alias(_) => None,
        })
        .collect::<Vec<_>>();
    eprintln!("{} fallbacks", fallbacks.len());
}
