//! Exploratory hackery

use fonts::Fonts;
use itertools::Itertools;

use icu_segmenter::GraphemeClusterSegmenter;

pub mod fonts;
pub mod fonts_xml;

pub fn graphemes(s: &str) -> Vec<&str> {
    GraphemeClusterSegmenter::new()
        .segment_str(s)
        .tuple_windows()
        .map(|(i, j)| &s[i..j])
        .collect()
}

pub fn itemize(s: &str, fonts: Fonts) {
    let familyset = fonts_xml::Familyset::from_fonts_xml();
    let fallbacks = familyset.fallbacks();
    eprintln!("{} fallbacks", fallbacks.len());
}
