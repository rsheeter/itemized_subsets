//! Access to font files

use std::{collections::HashMap, fs::{self, File}, io::Write, path::{Path, PathBuf}};

use regex::Regex;
use reqwest::blocking::Client as BlockingClient;
use smol_str::SmolStr;
use walkdir::WalkDir;

use crate::fonts_xml::{Family, Familyset};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontIdentifier {
    Filename(SmolStr),
    PostscriptName(SmolStr),
}

pub struct Fonts(HashMap<FontIdentifier, PathBuf>);

fn is_desirable_family(family_name: &str) -> bool {
    match family_name {
        // Android has 3 emoji families: COLR, CBDT, and flags. We just want one.
        v if v.starts_with("Noto Color Emoji ") => false,
        _ => true,
    }
}

impl Fonts {
    pub fn from_web(local_cache: &Path, families: &[&Family]) -> Self {

        let re = Regex::new(r"([a-z])([A-Z])").unwrap();
        let mut family_names = families.iter()
            .flat_map(|f| f.fonts.iter().map(|f| f.filename.as_str()))
            // Drop ext
            .map(|f| &f[0..f.len() - 4])
            // Something-Regular => Something
            .map(|f| f.split_once('-').map(|p| p.0).unwrap_or(f))
            // MyFamily-Regular => My Family
            .map(|f| re.replace_all(f, "$1 $2"))
            .collect::<Vec<_>>();
        family_names.sort();
        family_names.dedup();
        family_names.retain(|n| is_desirable_family(n));

        
        eprintln!("TODO: fetch {} families starting with {:?}", family_names.len(), &family_names[0..8]);

        let re = Regex::new(r"src: url[(]([^)]+)[)] format[(]'truetype'[)];").unwrap();
        let client = BlockingClient::new();
        for family_name in family_names {
            let mut local_file = local_cache.to_path_buf();
            local_file.push(family_name.replace(' ', "_"));
            if local_file.is_file() {
                continue;
            }

            // Fetch css
            let url = format!("https://fonts.googleapis.com/css2?family={}", family_name.replace(' ', "+"));
            eprintln!("{url} => {local_file:?}");
            let req = client.get(&url);
            let resp = req.send().and_then(|r| r.error_for_status());
            let Ok(resp) = resp else {
                eprintln!("Failed to fetch {url}: {resp:?}");
                continue;
            };
            
            let css = resp.text();
            let Ok(css) = css else {
                panic!("Failed to get text from response for {url}: {css:?}");
            };

            let Some(captures) = re.captures(&css) else {
                panic!("Failed to locate font url in response for {url}: {css:?}");
            };
            let Some(url) = captures.get(1) else {
                panic!("Failed to locate font url in captures for {url}: {css:?}");
            };

            // Fetch font binary
            let url = url.as_str();
            let req = client.get(url);
            let resp = req.send().and_then(|r| r.error_for_status());
            let Ok(resp) = resp else {
                panic!("Failed to fetch {url}: {resp:?}");
            };
            
            let bytes = resp.bytes();
            let Ok(bytes) = bytes else {
                panic!("Failed to get bytes from response for {url}: {bytes:?}");
            };

            eprintln!("Writing {} bytes to {:?}", bytes.len(), local_file);
            fs::write(&local_file, &*bytes).unwrap_or_else(|e| panic!("Unable to write to {local_file:?} {e:?}"));
        }

        Fonts(Default::default())
    }

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

            if map
                .insert(
                    FontIdentifier::Filename(filename.into()),
                    e.path().to_path_buf(),
                )
                .is_some()
            {
                eprintln!("Multiple files named {:?} :(", e.file_name());
            }
        }
        println!("{} font files", map.len());
        Self(map)
    }

    pub fn contains(&self, identifier: &FontIdentifier) -> bool {
        self.0.contains_key(identifier)
    }
}
