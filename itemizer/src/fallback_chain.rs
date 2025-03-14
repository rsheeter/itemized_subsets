//! A font fallback chain, against which one might itemize some text.

use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap, HashSet},
};

use icu_segmenter::GraphemeClusterSegmenter;
use itertools::Itertools;
use smol_str::SmolStr;

use crate::{Error, Run};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FontIdx(usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Family {
    pub family_name: SmolStr,
    pub lang: Option<SmolStr>,
    pub codepoints: BTreeSet<u32>,
}

#[derive(Debug)]
struct CodepointMapping {
    start: u32,
    end: u32,
    font: FontIdx,
}

pub struct FallbackChain {
    name: SmolStr,
    families: Vec<Family>,
    // No overlaps. Sorted.
    mappings: Vec<CodepointMapping>,
}

impl FallbackChain {
    pub fn for_fonts(
        name: &str,
        mut families: Vec<Family>,
        codepoints: impl Fn(&Family) -> HashSet<u32>,
    ) -> Self {
        let codepoints = families.iter().map(|f| codepoints(f)).collect::<Vec<_>>();

        let font_indices = (0..families.len())
            .map(|i| (&families[i], FontIdx(i)))
            .collect::<HashMap<_, _>>();

        // Map each codepoint to the families that support it
        let mut families_by_cp = HashMap::<u32, HashSet<&Family>>::new();
        for (font, codepoints) in families.iter().zip(codepoints.iter()) {
            for cp in codepoints {
                families_by_cp.entry(*cp).or_default().insert(font);
            }
        }

        // Match Android in preferring the head to all alternatives
        for support_group in families_by_cp.values_mut() {
            if support_group.contains(&families[0]) {
                support_group.retain(|f| *f == &families[0]);
            }
        }

        let mut distinct = 0;
        let mut unambiguous_conflict = 0;
        let mut conflict_groups = HashMap::<Vec<_>, HashSet<u32>>::new();
        for (cp, fonts) in families_by_cp.iter() {
            if fonts.len() == 1 {
                distinct += 1;
                continue;
            }
            // if we don't have lang tags there is no reason to alter priority
            if !fonts.iter().any(|f| f.lang.is_some()) {
                unambiguous_conflict += 1;
                continue;
            }
            // Conflicts!
            let mut conflict: Vec<_> = fonts.into_iter().collect();
            conflict.sort();
            conflict_groups.entry(conflict).or_default().insert(*cp);
        }

        for (families, codepoints) in conflict_groups.iter() {
            let mut cp_str = codepoints
                .iter()
                .map(|cp| format!("0x{cp:04x}"))
                .collect::<Vec<_>>();
            cp_str.sort();
            let cp_str = cp_str.into_iter().join(",");
            eprintln!(
                "{} {}",
                families.iter().map(|f| f.family_name.as_str()).join(","),
                cp_str
            );
        }

        eprintln!(
            "{distinct}/{} ({:.1}%) codepoints map to exactly 1 family",
            families_by_cp.len(),
            100.0 * distinct as f32 / families_by_cp.len() as f32
        );
        eprintln!(
            "{unambiguous_conflict}/{} ({:.1}%) codepoints map to multiple families but one is the clear winner",
            families_by_cp.len(),
            100.0 * unambiguous_conflict as f32 / families_by_cp.len() as f32
        );

        eprintln!(
            "{} distinct groups of fonts with ambiguous codepoints",
            conflict_groups.len()
        );

        let mut unambiguous_cp = families_by_cp
            .iter()
            .filter_map(|(cp, families)| if families.len() == 1 { Some(*cp) } else { None })
            .collect::<Vec<_>>();
        unambiguous_cp.sort();
        let num_unambiguous = unambiguous_cp.len();

        let font_idx_for_cp = |cp| {
            *font_indices
                .get(families_by_cp.get(&cp).unwrap().iter().next().unwrap())
                .unwrap()
        };
        let mut mappings = vec![CodepointMapping {
            start: unambiguous_cp[0],
            end: unambiguous_cp[0],
            font: font_idx_for_cp(unambiguous_cp[0]),
        }];
        for cp in unambiguous_cp.into_iter().skip(1) {
            let font = font_idx_for_cp(cp);
            let curr = mappings.last_mut().unwrap();
            if curr.end + 1 == cp && curr.font == font {
                curr.end = cp;
            } else {
                mappings.push(CodepointMapping {
                    start: cp,
                    end: cp,
                    font,
                })
            }
        }

        eprintln!(
            "{} mappings for {} unambiguous cp",
            mappings.len(),
            num_unambiguous
        );

        for (family, codepoints) in families.iter_mut().zip(codepoints.into_iter()) {
            family.codepoints = codepoints.into_iter().collect();
        }

        FallbackChain {
            name: name.into(),
            families,
            mappings,
        }
    }

    // TODO: match Android, test as much
    fn score(family: &Family, lang: &str, grapheme: &str) -> i32 {
        // TODO: handle fe0f properly
        if grapheme
            .chars()
            .filter(|cp| (*cp as u32) != 0xfe0f)
            .any(|cp| !family.codepoints.contains(&(cp as u32)))
        {
            return i32::MIN;
        }
        let mut score = 0; // full support, no other clues
        if Some(lang) == family.lang.as_deref() {
            score = i32::MAX;
        }
        score
    }

    pub fn itemize<'chain, 'text>(
        &'chain self,
        text: &'text str,
        lang: &str,
        dest: &mut Vec<Run<'chain>>,
    ) -> Result<(), Error> {
        dest.clear();
        for (start, end) in GraphemeClusterSegmenter::new()
            .segment_str(text)
            .tuple_windows()
        {
            let grapheme = &text[start..end];
            let mut chars = grapheme.chars();
            let mut match_type = "unattempted";
            let Some(first) = chars.next() else {
                debug_assert!(false, "empty grapheme?!");
                continue;
            };

            let mut family: Option<&'chain Family> = None;

            if chars.next().is_none() {
                // Single char grapheme, see if exactly one family supports it
                family = self
                    .mappings
                    .binary_search_by(|m| match first as u32 {
                        first if m.start > first => Ordering::Greater,
                        first if m.end < first => Ordering::Less,
                        _ => Ordering::Equal,
                    })
                    .ok()
                    .map(|mapping_idx| &self.families[self.mappings[mapping_idx].font.0]);
                match_type = "jump";
            }

            if family.is_none() {
                // Walk the chain to find the best match that supports the entire grapheme
                let mut winner = &self.families[0];
                let mut score = Self::score(&winner, lang, grapheme);
                for candidate in self.families.iter().skip(1) {
                    let candidate_score = Self::score(candidate, lang, grapheme);
                    if candidate_score > score {
                        winner = candidate;
                        score = candidate_score;
                    }
                    if score == i32::MAX {
                        // can't beat that
                        break;
                    }
                }
                if score > i32::MIN {
                    family = Some(winner);
                    match_type = "walk";
                } else {
                    match_type = "walk_to_eof";
                }
            }

            if let Some(family) = family {
                if dest.is_empty() {
                    dest.push(Run {
                        family,
                        start,
                        end: start,
                    });
                }

                let mut curr = dest.last_mut().unwrap();
                let mut op = "continue";
                if curr.family == family && curr.end == start {
                    curr.end = end;
                } else {
                    dest.push(Run { family, start, end });
                    curr = dest.last_mut().unwrap();
                    op = "insert";
                }
                eprintln!(
                    "{grapheme} {match_type} => {} ({}..{}) {} ({op})",
                    &text[curr.start..curr.end],
                    curr.start,
                    curr.end,
                    curr.family.family_name
                );
            } else {
                eprintln!(
                    "{grapheme} ({} codepoints) {match_type}, failed",
                    grapheme.chars().count()
                );
            }
        }
        Ok(())
    }
}
