pub(crate) fn count_syllables(word: &str) -> usize {
    let lower = word.to_lowercase();
    let chars: Vec<char> = lower.chars().filter(|c| c.is_ascii_alphabetic()).collect();

    if chars.is_empty() {
        return 0;
    }

    let is_vowel = |c: char| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');

    let mut count = 0usize;
    let mut prev_vowel = false;
    for &c in &chars {
        let v = is_vowel(c);
        if v && !prev_vowel {
            count += 1;
        }
        prev_vowel = v;
    }

    // silent trailing 'e' — except for the "-le" ending after a consonant (e.g. "table").
    let n = chars.len();
    if n >= 2 && chars[n - 1] == 'e' {
        let is_consonant_le = n >= 3 && chars[n - 2] == 'l' && !is_vowel(chars[n - 3]);
        if !is_consonant_le && count > 1 {
            count -= 1;
        }
    }

    count.max(1)
}
