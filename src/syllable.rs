use crate::lang::Lang;
use crate::pyphen;
use std::collections::HashMap;
use std::sync::LazyLock;

static CMU_RAW: &str = include_str!("data/cmudict.txt");

static CMU: LazyLock<HashMap<String, usize>> = LazyLock::new(|| {
    let mut map: HashMap<String, usize> = HashMap::new();
    for line in CMU_RAW.lines() {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(word) = parts.next() else { continue };
        let Some(variant) = parts.next() else {
            continue;
        };
        if variant != "1" {
            continue;
        }
        let syllables = parts
            .filter(|p| p.chars().last().is_some_and(|c| c.is_ascii_digit()))
            .count();
        map.insert(word.to_ascii_lowercase(), syllables);
    }
    map
});

pub(crate) fn count_syllables(text: &str, lang: Lang) -> usize {
    if text.len() == 0 {
        return 0;
    }
    let lower: String = text.to_lowercase();

    if lower.is_empty() {
        return 0;
    }
    // CMUdict is deliberately not locale-sensitive: textstat's `get_cmudict`
    // keys on the language *root*, so en_GB and en_US share one pronunciation
    // dictionary. Only the fallback below varies by locale, which is why a word
    // in CMUdict scores the same in both — the British and American pattern
    // files disagree about e.g. `process`, but neither is ever consulted for it.
    if let Some(&n) = CMU.get(&lower) {
        return n.max(1);
    }

    // out of vocab fallback - hyphenation points + 1, as textstat does via pyphen
    pyphen::positions(&lower, lang).len() + 1
}
