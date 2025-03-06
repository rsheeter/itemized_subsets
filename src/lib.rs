//! Exploratory hackery
use itertools::Itertools;

use icu_segmenter::GraphemeClusterSegmenter;

pub(crate) mod fonts_xml;

pub fn graphemes(s: &str) -> Vec<&str> {
    GraphemeClusterSegmenter::new()
        .segment_str(s)
        .tuple_windows()
        .map(|(i, j)| &s[i..j])
        .collect()
}

pub fn itemize(s: &str) {
    fonts_xml::Familyset::from_fonts_xml();
}
