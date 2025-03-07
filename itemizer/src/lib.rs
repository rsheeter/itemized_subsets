//! Exploratory hackery

use itertools::Itertools;

use icu_segmenter::GraphemeClusterSegmenter;

pub fn graphemes(s: &str) -> Vec<&str> {
    GraphemeClusterSegmenter::new()
        .segment_str(s)
        .tuple_windows()
        .map(|(i, j)| &s[i..j])
        .collect()
}

// pub fn itemize(fallbacks: Vec<&Family>, fonts: Fonts) {
//     eprintln!("{} fallbacks", fallbacks.len());
// }
