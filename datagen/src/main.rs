//! Generates data for itemizer
use std::{fs, path::{Path}};

use datagen::{font_binaries::{FontIdentifier, Fonts}, fonts_xml::Familyset};

fn main() {
    let local_cache = Path::new("/tmp/local_fonts");
    if !local_cache.is_dir() {
        fs::create_dir(local_cache).expect("To create local fonts dir");
    }
    let familyset = Familyset::bundled_fonts_xml();
    let fallbacks = familyset.fallbacks();
    let fonts = Fonts::from_web(local_cache, &fallbacks);
    

    let mut contains = 0;
    let mut missing = 0;
    for fallback in fallbacks {
        for font in fallback.fonts.iter() {
            if fonts.contains(&FontIdentifier::Filename(font.filename.as_str().into())) {
                contains += 1;
            } else {
                missing += 1;
                //eprintln!("Unable to locate {}", font.filename);
            }
        }
    }

    println!("{contains}/{} fallback fonts located", contains + missing);
}
