//! Port of the parts of `pyphen` that textstat actually uses.
//!
//! textstat only ever calls `Pyphen(lang=...).positions(word)` and takes
//! `len(positions) + 1` as the syllable count for out-of-vocabulary words. That
//! means we can skip pyphen's `DataInt` / `AlternativeParser` machinery: those
//! only describe *how* to break a word at a position, never *where* the breaks
//! are. The odd/even value at each position is identical either way. (Neither
//! English dictionary contains a nonstandard alternative anyway — no line in
//! either file has both a `/` and an `=`.)
//!
//! So this is just: parse the Liang patterns out of the `.dic` file, then
//! max-merge them over the word and keep the odd positions.
//!
//! The two English dictionaries are structurally identical — same header keys,
//! same UTF-8 encoding, both carrying apostrophes and Unicode `ﬁ`-ligatures —
//! so the locale only picks which pattern set to run, never how to parse it.

use crate::lang::Lang;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// pyphen's `left`/`right` defaults: a break must leave at least this many
/// characters on each side of the word.
///
/// These stay 2 for both locales. Both `.dic` files declare `RIGHTHYPHENMIN 3`
/// in their headers, but pyphen never reads that — it takes `left`/`right` from
/// its constructor arguments and lists the header keys among `ignored`, and
/// textstat constructs `Pyphen(lang=lang)` without overriding either.
const LEFT: isize = 2;
const RIGHT: isize = 2;

const IGNORED_PREFIXES: [&str; 6] = [
    "%",
    "#",
    "LEFTHYPHENMIN",
    "RIGHTHYPHENMIN",
    "COMPOUNDLEFTHYPHENMIN",
    "COMPOUNDRIGHTHYPHENMIN",
];

static HEX_ESCAPE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\^{2}([0-9a-f]{2})").unwrap());

struct HyphDict {
    /// tags -> (offset of the first non-zero value, the non-zero-bounded values)
    patterns: HashMap<String, (usize, Vec<u8>)>,
    /// Longest key, in `char`s (not bytes).
    maxlen: usize,
}

/// Parse the Liang patterns out of one `hyph_*.dic` file.
fn parse_dict(raw: &str) -> HyphDict {
    let mut patterns: HashMap<String, (usize, Vec<u8>)> = HashMap::new();

    // Line 1 is the encoding declaration.
    for line in raw.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() || IGNORED_PREFIXES.iter().any(|p| line.starts_with(p)) {
            continue;
        }

        let line = HEX_ESCAPE.replace_all(line, |caps: &regex::Captures| {
            let byte = u8::from_str_radix(&caps[1], 16).expect("regex guarantees two hex digits");
            (byte as char).to_string()
        });

        // A nonstandard hyphen alternative (`pattern/change,index,cut`) changes
        // how a break is rendered, not where it falls, so drop it.
        let pattern = match line.split_once('/') {
            Some((before, _)) if line.contains('=') => before,
            _ => &line,
        };

        // Digits annotate the gap *before* the character that follows them, so
        // `values[k]` is the priority of the gap in front of `tags[k]`, with one
        // extra slot for a digit trailing the final character.
        let mut tags = String::new();
        let mut values: Vec<u8> = Vec::new();
        values.push(0);
        for c in pattern.chars() {
            if let Some(d) = c.to_digit(10) {
                *values.last_mut().expect("values is never empty") = d as u8;
            } else {
                tags.push(c);
                values.push(0);
            }
        }

        // Patterns that say nothing can't influence the max-merge.
        let Some(end) = values.iter().rposition(|&v| v != 0) else {
            continue;
        };
        let start = values
            .iter()
            .position(|&v| v != 0)
            .expect("a last non-zero implies a first");

        patterns.insert(tags, (start, values[start..=end].to_vec()));
    }

    let maxlen = patterns
        .keys()
        .map(|k| k.chars().count())
        .max()
        .expect("the dictionary is not empty");

    HyphDict { patterns, maxlen }
}

// Parsed independently and on demand, mirroring pyphen's per-file `hdcache`:
// scoring only ever in one locale never pays to parse the other's ~107 KB of
// patterns.
static HYPH_EN_US: LazyLock<HyphDict> =
    LazyLock::new(|| parse_dict(include_str!("data/hyph_en_US.dic")));
static HYPH_EN_GB: LazyLock<HyphDict> =
    LazyLock::new(|| parse_dict(include_str!("data/hyph_en_GB.dic")));

/// Positions in `word` where a hyphen may be inserted, as `char` offsets.
///
/// Mirrors `Pyphen.positions`: the Liang max-merge over `HyphDict.positions`,
/// then dropping breaks that sit too close to either end of the word.
pub(crate) fn positions(word: &str, lang: Lang) -> Vec<usize> {
    let dict: &HyphDict = match lang {
        Lang::EnUs => &HYPH_EN_US,
        Lang::EnGb => &HYPH_EN_GB,
    };

    // Python indexes `str` by character, so we must too — otherwise a non-ASCII
    // word would produce byte offsets and diverge.
    let word_len = word.chars().count() as isize;
    let mut pointed: Vec<char> = Vec::with_capacity(word_len as usize + 2);
    pointed.push('.');
    pointed.extend(word.to_lowercase().chars());
    pointed.push('.');

    let mut references = vec![0u8; pointed.len() + 1];
    // Reused across lookups so matching a pattern doesn't allocate a key each time.
    let mut key = String::new();

    for i in 0..pointed.len().saturating_sub(1) {
        let stop = (i + dict.maxlen).min(pointed.len()) + 1;
        for j in i + 1..stop {
            key.clear();
            key.extend(&pointed[i..j]);
            let Some((offset, values)) = dict.patterns.get(key.as_str()) else {
                continue;
            };
            // Bounded by construction: offset + values.len() <= key.len() + 1,
            // so i + offset + values.len() <= j + 1 <= references.len().
            for (k, &value) in values.iter().enumerate() {
                let slot = &mut references[i + offset + k];
                *slot = (*slot).max(value);
            }
        }
    }

    references
        .iter()
        .enumerate()
        .filter(|(_, r)| *r % 2 == 1)
        .map(|(i, _)| i as isize - 1)
        .filter(|&p| LEFT <= p && p <= word_len - RIGHT)
        .map(|p| p as usize)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Expectations are pinned to real pyphen 0.17.2 output:
    /// `Pyphen(lang=...).positions(word)` for each locale.
    const CASES: &[(&str, &[usize], &[usize])] = &[
        // word           en_US            en_GB
        ("powerhouse", &[3, 5], &[5]),
        ("molecules", &[3, 4], &[2]),
        ("colour", &[], &[3]),
        ("organisation", &[2, 5, 6, 8], &[2, 5, 8]),
        ("analyse", &[2, 3], &[3]),
        ("photosynthesis", &[3, 5, 8, 11], &[3, 5, 8, 12]),
        ("readability", &[4, 8, 9], &[4, 6, 8]),
    ];

    #[test]
    fn matches_pyphen_per_locale() {
        for &(word, us, gb) in CASES {
            assert_eq!(positions(word, Lang::EnUs), us, "{word} (en_US)");
            assert_eq!(positions(word, Lang::EnGb), gb, "{word} (en_GB)");
        }
    }

    /// The locales really do disagree — guards against both statics ending up
    /// pointed at the same file.
    #[test]
    fn locales_are_not_the_same_dictionary() {
        assert_ne!(positions("colour", Lang::EnUs), positions("colour", Lang::EnGb));
    }
}
