//! Access to font files

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

pub struct Fonts(HashMap<String, PathBuf>);

impl Fonts {
    pub fn from_dir(d: &str) -> Self {
        let mut map = HashMap::new();
        for e in WalkDir::new(d).into_iter() {
            let Ok(e) = e else {
                eprintln!("Walk error {e:?}");
                continue;
            };
            if !e.path().is_file() {
                continue;
            }
            let Some(filename) = e.file_name().to_str().map(|s| s.to_ascii_lowercase()) else {
                eprintln!("Non-unicode filename? - skipping {e:?}");
                continue;
            };
            if !(filename.ends_with(".ttf") || filename.ends_with(".otf")) {
                continue;
            }
            if map.insert(filename, e.path().to_path_buf()).is_some() {
                eprintln!("Multiple files named {:?} :(", e.file_name());
            }
        }
        println!("{} font files", map.len());
        Self(map)
    }
}
