//! Numbers: a French dictation that comes out in words ("vingt-cinq",
//! "quatre-vingt-dix-huit", "deux mille vingt-six") becomes digits ("25",
//! "98", "2026"). Parakeet writes numbers out in full for French, and the
//! Android app has no rewriting model behind it, so the text steps turn them
//! into digits themselves.
//!
//! Only French is handled. Words are matched on their own (word boundaries,
//! case, and accents ignored) and joined across spaces, hyphens, and "et", so
//! "quatre-vingt-dix-huit", "quatre vingt dix huit", and "soixante et onze"
//! all read. A lone "un" or "une" is the article and stays: "un livre" must
//! not become "1 livre". A compound that carries a non-number tail is left
//! whole ("vingt-cinquième", "dix-septième").
//!
//! "pour cent" follows its number ("trois pour cent" is "3 %"), and "virgule"
//! joins two numbers with a comma ("trois virgule quatorze" is "3,14").
//! Matching ignores case and the accents of the models' spelling, so "Zéro"
//! and "zéro" read the same.

use super::spoken_formatting::fold;

/// A French number word and what it contributes.
const WORDS: &[(&str, u64)] = &[
    ("zero", 0),
    ("un", 1),
    ("une", 1),
    ("deux", 2),
    ("trois", 3),
    ("quatre", 4),
    ("cinq", 5),
    ("six", 6),
    ("sept", 7),
    ("huit", 8),
    ("neuf", 9),
    ("dix", 10),
    ("onze", 11),
    ("douze", 12),
    ("treize", 13),
    ("quatorze", 14),
    ("quinze", 15),
    ("seize", 16),
    ("vingt", 20),
    ("vingts", 20),
    ("trente", 30),
    ("quarante", 40),
    ("cinquante", 50),
    ("soixante", 60),
    ("cent", 100),
    ("cents", 100),
    ("mille", 1_000),
    ("million", 1_000_000),
    ("millions", 1_000_000),
    ("milliard", 1_000_000_000),
    ("milliards", 1_000_000_000),
];

/// What a word does to the number being read.
#[derive(Clone, Copy)]
enum Kind {
    Unit(u64),
    Tens(u64),
    Hundred,
    Thousand,
    Million(u64),
}

/// The word's role, or `None` for "et" and everything that is not a number.
fn kind_of(word: &str) -> Option<Kind> {
    let value = WORDS.iter().find(|(known, _)| *known == word)?.1;
    Some(match value {
        0..=19 => Kind::Unit(value),
        20 | 30 | 40 | 50 | 60 => Kind::Tens(value),
        100 => Kind::Hundred,
        1_000 => Kind::Thousand,
        scale => Kind::Million(scale),
    })
}

/// The value read so far, with the scale words folded into `total`.
#[derive(Default)]
struct State {
    total: u64,
    current: u64,
    tokens: usize,
    hundred_used: bool,
    thousand_used: bool,
    et_used: bool,
}

impl State {
    fn apply(&mut self, kind: Kind) -> bool {
        match kind {
            Kind::Unit(unit) => self.unit(unit),
            Kind::Tens(tens) => self.tens(tens),
            Kind::Hundred => self.hundred(),
            Kind::Thousand => self.thousand(),
            Kind::Million(scale) => self.million(scale),
        }
    }

    /// The largest unit a tens can still take: "soixante-dix-neuf" goes to 19,
    /// "vingt-neuf" stops at 9.
    fn unit_limit(&self) -> u64 {
        match self.current % 100 {
            0 | 60 | 80 => 19,
            _ => 9,
        }
    }

    fn unit(&mut self, unit: u64) -> bool {
        if self.tokens == 0 || self.current == 0 {
            self.current = unit;
            self.tokens += 1;
            return true;
        }
        if self.current % 10 == 0 && unit <= self.unit_limit() {
            self.current += unit;
            self.tokens += 1;
            return true;
        }
        false
    }

    fn tens(&mut self, tens: u64) -> bool {
        // "quatre-vingt" is 4 × 20, not 4 then 20: the only multiplication a
        // tens word takes.
        if tens == 20 && self.current % 10 == 4 {
            self.current += 76;
            self.tokens += 1;
            return true;
        }
        if self.current == 0 || self.current % 100 == 0 {
            self.current += tens;
            self.tokens += 1;
            return true;
        }
        false
    }

    fn hundred(&mut self) -> bool {
        if self.hundred_used || self.current >= 100 {
            return false;
        }
        self.current = match self.current {
            0 => 100,
            1..=9 => self.current * 100,
            _ => return false,
        };
        self.hundred_used = true;
        self.tokens += 1;
        true
    }

    fn thousand(&mut self) -> bool {
        if self.thousand_used {
            return false;
        }
        self.total += self.current.max(1) * 1_000;
        self.current = 0;
        self.thousand_used = true;
        self.hundred_used = false;
        self.tokens += 1;
        true
    }

    fn million(&mut self, scale: u64) -> bool {
        // "des millions" is a noun, not a number: a scale needs a number.
        if self.current == 0 {
            return false;
        }
        self.total += self.current * scale;
        self.current = 0;
        self.thousand_used = false;
        self.hundred_used = false;
        self.tokens += 1;
        true
    }

    fn value(&self) -> u64 {
        self.total + self.current
    }
}

/// A number found in the text: its value, where it ends, and whether it is
/// just "un"/"une" (the article, unless a decimal or "pour cent" follows).
struct Found {
    value: u64,
    end: usize,
    single_un: bool,
}

/// `transcript` with its French number words written as digits.
pub fn apply(transcript: &str) -> String {
    apply_for(transcript, "en")
}

/// `transcript` with the number words of `language` written as digits.
pub fn apply_for(transcript: &str, language: &str) -> String {
    if !language.starts_with("fr") {
        return transcript.to_owned();
    }
    let chars: Vec<char> = transcript.chars().collect();
    let mut out = String::with_capacity(transcript.len());
    let mut index = 0;
    while index < chars.len() {
        let word_start = index == 0 || !chars[index - 1].is_alphanumeric();
        if word_start && chars[index].is_alphabetic() {
            if let Some(found) = number_at(&chars, index) {
                let decimal = decimal_at(&chars, found.end);
                let percent = percent_end(&chars, found.end);
                let after_pour = previous_word(&chars, index).is_some_and(|word| word == "pour");
                if !(found.single_un && decimal.is_none() && percent.is_none())
                    && !(after_pour && starts_with_cent(&chars, index, found.end))
                {
                    out.push_str(&found.value.to_string());
                    if let Some((fraction, end)) = decimal {
                        out.push(',');
                        out.push_str(&fraction.to_string());
                        index = end;
                    } else if let Some(end) = percent {
                        out.push_str(" %");
                        index = end;
                    } else {
                        index = found.end;
                    }
                    continue;
                }
            }
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

/// The number whose first word starts at `start`, read greedily.
fn number_at(chars: &[char], start: usize) -> Option<Found> {
    let tokens = tokens_at(chars, start);
    if tokens.is_empty() {
        return None;
    }
    let mut state = State::default();
    let mut end = start;
    let mut index = 0;
    while index < tokens.len() {
        let (word, word_end) = &tokens[index];
        if word == "et" {
            let Some((next, next_end)) = tokens.get(index + 1) else {
                break;
            };
            let Some(Kind::Unit(unit)) = kind_of(next) else {
                break;
            };
            let valid = (1..=11).contains(&unit)
                && state.tokens > 0
                && !state.et_used
                && state.current % 10 == 0;
            if !valid || !state.unit(unit) {
                break;
            }
            state.et_used = true;
            end = *next_end;
            index += 2;
            continue;
        }
        let Some(kind) = kind_of(word) else {
            break;
        };
        if !state.apply(kind) {
            break;
        }
        end = *word_end;
        index += 1;
    }
    if state.tokens == 0 {
        return None;
    }
    Some(Found {
        value: state.value(),
        end,
        single_un: state.tokens == 1 && state.value() == 1,
    })
}

/// The number words starting at `start`, as normalised words with their end
/// offsets. Stops at the first word that is not part of a number, and gives
/// up when an unknown word is glued to one by a hyphen ("vingt-cinquième").
fn tokens_at(chars: &[char], start: usize) -> Vec<(String, usize)> {
    let mut tokens = Vec::new();
    let mut position = start;
    loop {
        let word_end = read_word(chars, position);
        if word_end == position {
            break;
        }
        let word = normalise(&chars[position..word_end]);
        if kind_of(&word).is_none() && word != "et" {
            break;
        }
        tokens.push((word, word_end));
        let Some(gap) = read_gap(chars, word_end) else {
            break;
        };
        let next_end = read_word(chars, gap.end);
        if next_end == gap.end {
            if gap.glued {
                return Vec::new();
            }
            break;
        }
        let next = normalise(&chars[gap.end..next_end]);
        if kind_of(&next).is_none() && next != "et" {
            if gap.glued {
                return Vec::new();
            }
            break;
        }
        position = gap.end;
    }
    tokens
}

/// A separator between two number words: spaces, hyphens, or both.
struct Gap {
    end: usize,
    /// A hyphen with no space around it joins the words into one.
    glued: bool,
}

fn read_gap(chars: &[char], start: usize) -> Option<Gap> {
    let mut index = start;
    let mut space = false;
    let mut hyphen = false;
    while let Some(&c) = chars.get(index) {
        if c.is_whitespace() && c != '\n' {
            space = true;
        } else if matches!(c, '-' | '\u{2010}' | '\u{2011}' | '\u{2013}') {
            hyphen = true;
        } else {
            break;
        }
        index += 1;
    }
    (index > start).then_some(Gap {
        end: index,
        glued: hyphen && !space,
    })
}

fn read_word(chars: &[char], start: usize) -> usize {
    let mut index = start;
    while chars.get(index).is_some_and(|c| c.is_alphabetic()) {
        index += 1;
    }
    index
}

fn normalise(word: &[char]) -> String {
    word.iter().map(|&c| fold(c)).collect()
}

/// The whole word before `index`, when one is separated from it by spaces.
fn previous_word(chars: &[char], index: usize) -> Option<String> {
    let mut start = index;
    while start > 0 && chars[start - 1].is_whitespace() {
        start -= 1;
    }
    let end = start;
    while start > 0 && chars[start - 1].is_alphabetic() {
        start -= 1;
    }
    (start < end).then(|| normalise(&chars[start..end]))
}

fn starts_with_cent(chars: &[char], start: usize, end: usize) -> bool {
    let first_end = read_word(chars, start).min(end);
    matches!(
        normalise(&chars[start..first_end]).as_str(),
        "cent" | "cents"
    )
}

/// Where "pour cent" ends after `from`, when it is there.
fn percent_end(chars: &[char], from: usize) -> Option<usize> {
    let after_pour = word_after(chars, from, "pour")?;
    word_after(chars, after_pour, "cent")
}

/// The fraction after "virgule": its value and where the fraction's number
/// ends.
fn decimal_at(chars: &[char], from: usize) -> Option<(u64, usize)> {
    let after_virgule = word_after(chars, from, "virgule")?;
    let gap = read_gap(chars, after_virgule)?;
    let found = number_at(chars, gap.end)?;
    (found.value <= 999).then_some((found.value, found.end))
}

/// Where the whole word `expected` ends after `from`, when it comes next.
fn word_after(chars: &[char], from: usize, expected: &str) -> Option<usize> {
    let gap = read_gap(chars, from)?;
    let end = read_word(chars, gap.end);
    let word = normalise(&chars[gap.end..end]);
    (end > gap.end && word == expected).then_some(end)
}

#[cfg(test)]
mod tests {
    use super::{apply, apply_for};

    #[test]
    fn units_tens_and_hundreds_become_digits() {
        assert_eq!(apply_for("J'ai vingt-cinq ans.", "fr"), "J'ai 25 ans.");
        assert_eq!(apply_for("dix-huit et dix-neuf", "fr"), "18 et 19");
        assert_eq!(apply_for("soixante et onze", "fr"), "71");
        assert_eq!(apply_for("quatre-vingt-neuf", "fr"), "89");
        assert_eq!(apply_for("quatre-vingt-dix-huit", "fr"), "98");
        assert_eq!(apply_for("cent cinquante-trois", "fr"), "153");
        assert_eq!(apply_for("deux cents", "fr"), "200");
        assert_eq!(apply_for("deux cent un", "fr"), "201");
    }

    #[test]
    fn thousands_and_millions_become_digits() {
        assert_eq!(apply_for("deux mille vingt-six", "fr"), "2026");
        assert_eq!(
            apply_for("mille neuf cent quatre-vingt-dix-huit", "fr"),
            "1998"
        );
        assert_eq!(apply_for("deux mille cinq cents", "fr"), "2500");
        assert_eq!(apply_for("un million trois cent mille", "fr"), "1300000");
        assert_eq!(apply_for("mille et une nuits", "fr"), "1001 nuits");
    }

    #[test]
    fn hyphens_spaces_and_et_all_read() {
        assert_eq!(apply_for("quatre vingt dix huit", "fr"), "98");
        assert_eq!(apply_for("vingt-et-un", "fr"), "21");
        assert_eq!(apply_for("trente et une", "fr"), "31");
        assert_eq!(apply_for("soixante-dix-sept", "fr"), "77");
    }

    #[test]
    fn the_article_un_and_une_stay() {
        for text in ["un livre", "une heure", "Il en veut un."] {
            assert_eq!(apply_for(text, "fr"), text);
        }
        assert_eq!(apply_for("une idée de deux", "fr"), "une idée de 2");
    }

    #[test]
    fn compounds_with_a_non_number_tail_stay_whole() {
        for text in ["vingt-cinquième", "dix-septième", "un deuxième essai"] {
            assert_eq!(apply_for(text, "fr"), text);
        }
        assert_eq!(apply_for("Mille excuses", "fr"), "1000 excuses");
    }

    #[test]
    fn percent_and_decimals_follow_their_number() {
        assert_eq!(apply_for("trois pour cent", "fr"), "3 %");
        assert_eq!(apply_for("dix pour cent de plus", "fr"), "10 % de plus");
        assert_eq!(apply_for("un pour cent", "fr"), "1 %");
        assert_eq!(apply_for("cent pour cent", "fr"), "100 %");
        assert_eq!(apply_for("trois virgule quatorze", "fr"), "3,14");
        assert_eq!(apply_for("zéro virgule cinq", "fr"), "0,5");
    }

    #[test]
    fn punctuation_between_numbers_keeps_them_apart() {
        assert_eq!(
            apply_for("Il en faut deux, trois ou quatre.", "fr"),
            "Il en faut 2, 3 ou 4."
        );
        assert_eq!(apply_for("dix sur dix", "fr"), "10 sur 10");
    }

    #[test]
    fn other_languages_are_untouched() {
        for text in ["twenty-five", "vingt-cinq", "un livre"] {
            assert_eq!(apply(text), text);
            assert_eq!(apply_for(text, "en"), text);
            assert_eq!(apply_for(text, "de"), text);
        }
    }

    #[test]
    fn plain_text_is_untouched() {
        for text in [
            "Le mois de mai arrive.",
            "deuxième",
            "un pourboire",
            "des millions de gens",
            "",
        ] {
            assert_eq!(apply_for(text, "fr"), text);
        }
    }
}
