//! Access to font files

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::{self, File},
    path::{Path, PathBuf},
};

use memmap2::Mmap;
use regex::Regex;
use reqwest::blocking::Client as BlockingClient;
use skrifa::{FontRef, MetadataProvider};
use smol_str::SmolStr;
use walkdir::WalkDir;

use crate::fonts_xml::Family;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Filename(pub SmolStr);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FamilyName(pub SmolStr);

impl Display for Filename {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl Display for FamilyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl From<&Filename> for FamilyName {
    fn from(filename: &Filename) -> Self {
        let filename = filename.0.as_str();

        // Drop ext
        let filename = &filename[0..filename.len() - 4];

        // Something-Regular => Something or Something-VF => Something
        let filename = filename.split_once('-').map(|p| p.0).unwrap_or(filename);

        // MyFamily-Regular => My Family
        let re = Regex::new(r"([a-z])([A-Z])").unwrap();
        FamilyName(re.replace_all(filename, "$1 $2").into())
    }
}

/// Makes font binaries available for data generation
pub struct FontBinaries {
    local_files: HashMap<Filename, PathBuf>,
    by_family_name: HashMap<FamilyName, Filename>,
}

impl FontBinaries {
    fn new(local_files: HashMap<Filename, PathBuf>) -> Self {
        let by_family_name = local_files.keys().map(|f| (f.into(), f.clone())).collect();
        Self {
            local_files,
            by_family_name,
        }
    }

    pub fn from_web(local_cache: &Path, families: &[&Family]) -> Self {
        let local_files = families
            .iter()
            .map(|family| {
                for font in family.fonts.iter() {
                    debug_assert!(!font.filename.contains("CJK"), "{family:#?}");
                }
                family
            })
            .flat_map(|f| f.fonts.iter().map(|f| f.filename.as_str()))
            .map(|filename| {
                let filename = Filename(filename.into());
                let family_name: FamilyName = (&filename).into();
                (family_name, filename)
            })
            .map(|(family_name, filename)| {
                let mut local_file = local_cache.to_path_buf();
                local_file.push(family_name.0.replace(' ', "_"));
                (filename, local_file.clone())
            })
            .collect::<HashMap<_, _>>();

        let re = Regex::new(r"src: url[(]([^)]+)[)] format[(]'truetype'[)];").unwrap();
        let client = BlockingClient::new();
        for (filename, local_file) in local_files.iter() {
            if local_file.is_file() {
                continue;
            }
            let family_name: FamilyName = filename.into();

            // Fetch css
            let url = format!(
                "https://fonts.googleapis.com/css2?family={}",
                family_name.0.replace(' ', "+")
            );
            eprintln!("{family_name} => {local_file:?}");
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
            fs::write(&local_file, &*bytes)
                .unwrap_or_else(|e| panic!("Unable to write to {local_file:?} {e:?}"));
        }

        Self::new(local_files)
    }

    pub fn from_dir(d: &str) -> Self {
        let mut local_files = HashMap::new();
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

            if local_files
                .insert(Filename(filename.into()), e.path().to_path_buf())
                .is_some()
            {
                eprintln!("Multiple files named {:?} :(", e.file_name());
            }
        }
        println!("{} font files", local_files.len());
        Self::new(local_files)
    }

    pub fn local_file(&self, filename: &Filename) -> Option<&Path> {
        self.local_files.get(filename).map(PathBuf::as_path)
    }

    pub fn filename(&self, family_name: &FamilyName) -> Option<&Filename> {
        self.by_family_name.get(family_name)
    }

    pub fn codepoints(&self, filename: &Filename) -> Option<HashSet<u32>> {
        let Some(path) = self.local_file(filename) else {
            return None;
        };
        if !path.is_file() {
            return None;
        }
        let file = File::open(&path).unwrap_or_else(|e| panic!("Unable to read {path:?}: {e}"));
        let mmap =
            unsafe { Mmap::map(&file).unwrap_or_else(|e| panic!("Unable to mmap {path:?}: {e}")) };
        let font = FontRef::new(&*mmap)
            .unwrap_or_else(|e| panic!("Unable to create a fontref for {path:?}: {e}"));
        Some(
            font.charmap()
                .mappings()
                .into_iter()
                .map(|(cp, _)| cp)
                .collect(),
        )
    }
}
