//! Generates data for itemizer
use std::{fs, iter::once, path::Path};

use datagen::{
    font_binaries::{FamilyName, Filename, FontBinaries},
    fonts_xml::{Familyset, Font, Style},
};
use itemizer::fallback_chain::FallbackChain;

fn named_chain(familyset: &Familyset, font_binaries: &FontBinaries, head: &str) -> FallbackChain {
    let unwantedness = |font: &Font| {
        // Having the specified fallback name is best
        // Then no fallback name
        // Worst of all, the wrong fallback name
        let mut score = match font.fallback_for.as_deref() {
            Some(v) if v == head => 0,
            None => 10000,
            Some(_) => 100000,
        };
        if font.style == Style::Italic {
            score += 1000;
        }

        // Prefer nearest 400, failing that higher is better
        score += match font.weight - 400.0 {
            0.0 => 0,
            v if v > 0.0 => v as u32,
            v => (v + 50.0) as u32,
        };

        score
    };

    let Some(sans) = familyset.named(head) else {
        panic!("Unable to locate {head}");
    };
    let fonts = once(sans)
        .chain(familyset.fallbacks())
        .map(|family| {
            // Pick the best font from each family
            let font = family
                .fonts
                .iter()
                .reduce(|acc, e| {
                    if unwantedness(acc) <= unwantedness(e) {
                        acc
                    } else {
                        e
                    }
                })
                .unwrap_or_else(|| panic!("No family should be fontless! {family:?}"));
            let filename = Filename((&font.filename).into());
            let family_name: FamilyName = (&filename).into();
            itemizer::fallback_chain::Family {
                family_name: family_name.0,
                lang: family.lang.as_deref().map(|s| s.into()),
            }
        })
        .collect::<Vec<_>>();

    FallbackChain::for_fonts(head, fonts, |font| {
        font_binaries
            .filename(&FamilyName(font.family_name.clone()))
            .and_then(|filename| font_binaries.codepoints(filename))
            .unwrap_or_default()
    })
}

fn main() {
    let local_cache = Path::new("/tmp/local_fonts");
    if !local_cache.is_dir() {
        fs::create_dir(local_cache).expect("To create local fonts dir");
    }
    let familyset = Familyset::fonts_xml_for_googlefonts();
    let fallbacks = familyset.fallbacks();
    let fonts = FontBinaries::from_web(local_cache, &fallbacks);

    let mut contains = 0;
    let mut missing = 0;
    for fallback in fallbacks {
        for font in fallback.fonts.iter() {
            let Some(local_file) = fonts.local_file(&Filename(font.filename.as_str().into()))
            else {
                missing += 1;
                eprintln!("No entry for {}", font.filename);
                continue;
            };
            if local_file.is_file() {
                contains += 1;
            } else {
                missing += 1;
                eprintln!("{} local {:?} not found", font.filename, local_file);
            }
        }
    }

    println!("{contains}/{} fallback fonts located", contains + missing);

    let sans_chain = named_chain(&familyset, &fonts, "sans-serif");

    let text = "Hello 世界 ❤️‍🔥";
    let mut dest = Vec::new();
    sans_chain.itemize(text, "und-Latn", &mut dest).unwrap();
    eprintln!("Runs for {text}");
    for run in dest {
        eprintln!("{run:?}");
    }
}
