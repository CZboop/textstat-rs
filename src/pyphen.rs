//! Port of the parts of `pyphen` that textstat actually uses.
//!
//! textstat only ever calls `Pyphen(lang="en_US").positions(word)` and takes
//! `len(positions) + 1` as the syllable count for out-of-vocabulary words. That
//! means we can skip pyphen's `DataInt` / `AlternativeParser` machinery: those
//! only describe *how* to break a word at a position, never *where* the breaks
//! are. The odd/even value at each position is identical either way.
//!
//! So this is just: parse the Liang patterns out of the `.dic` file, then
//! max-merge them over the word and keep the odd positions.

use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static HYPH_RAW: &str = include_str!("data/hyph_en_US.dic");

/// pyphen's `left`/`right` defaults: a break must leave at least this many
/// characters on each side of the word.
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

static HYPH_DICT: LazyLock<HyphDict> = LazyLock::new(|| {
    let mut patterns: HashMap<String, (usize, Vec<u8>)> = HashMap::new();

    // Line 1 is the encoding declaration.
    for line in HYPH_RAW.lines().skip(1) {
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
});

/// Positions in `word` where a hyphen may be inserted, as `char` offsets.
///
/// Mirrors `Pyphen.positions`: the Liang max-merge over `HyphDict.positions`,
/// then dropping breaks that sit too close to either end of the word.
pub(crate) fn positions(word: &str) -> Vec<usize> {
    let dict = &*HYPH_DICT;

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
