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

pub(crate) fn count_syllables(text: &str) -> usize {
    if text.len() == 0 {
        return 0;
    }
    let lower: String = text.to_lowercase();

    if lower.is_empty() {
        return 0;
    }
    if let Some(&n) = CMU.get(&lower) {
        return n.max(1);
    }

    // out of vocab fallback - hyphenation points + 1, as textstat does via pyphen
    pyphen::positions(&lower).len() + 1
}
