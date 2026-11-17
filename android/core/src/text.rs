//! The text steps a dictation goes through after transcription, in the order
//! Hex's desktop `prepare_transcript` uses: spoken formatting, number words,
//! spoken symbols, filler words, then text replacements.
//!
//! The number step is the Android app's own: Parakeet writes French numbers
//! out in full ("quatre-vingt-dix-huit"), and no rewriting model runs on the
//! phone to turn them into digits.

use serde::Deserialize;

use crate::dictation::{
    filler_words, number_words, spoken_formatting, spoken_symbols,
    text_replacements::{ReplacementSet, TextReplacement},
};

/// Which steps run. The app sends this as JSON; a missing field is off.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct TextSteps {
    pub spoken_formatting: bool,
    pub spoken_symbols: bool,
    pub filler_words: bool,
    /// ISO code of the dictation. Filler words apply only to "en".
    pub language: String,
    pub replacements: Vec<TextReplacement>,
}

pub fn process(transcript: &str, steps: &TextSteps) -> String {
    let mut text = transcript.to_owned();
    if steps.spoken_formatting {
        text = spoken_formatting::apply_for(&text, &steps.language);
    }
    text = number_words::apply_for(&text, &steps.language);
    if steps.spoken_symbols {
        text = spoken_symbols::apply_for(&text, &steps.language);
    }
    if let Some(stripped) = filler_words::strip_if(&text, &steps.language, steps.filler_words) {
        text = stripped;
    }
    if !steps.replacements.is_empty() {
        text = ReplacementSet::new(&steps.replacements).replace(&text);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_on() -> TextSteps {
        TextSteps {
            spoken_formatting: true,
            spoken_symbols: true,
            filler_words: true,
            language: "en".into(),
            replacements: vec![TextReplacement {
                matched_phrase: "hex droid".into(),
                output: "HexDroid".into(),
            }],
        }
    }

    #[test]
    fn off_by_default_leaves_text_alone() {
        let text = "um, new line hashtag hex droid";
        assert_eq!(process(text, &TextSteps::default()), text);
    }

    #[test]
    fn runs_every_step() {
        let out = process("Um, I like hex droid new line thanks", &all_on());
        assert!(!out.to_lowercase().contains("um"), "{out}");
        assert!(out.contains("HexDroid"), "{out}");
        assert!(out.contains('\n'), "{out}");
    }

    #[test]
    fn fillers_stay_outside_english() {
        let steps = TextSteps {
            language: "de".into(),
            ..all_on()
        };
        assert!(process("um das", &steps).contains("um"));
    }

    #[test]
    fn french_numbers_become_digits_before_the_symbols() {
        let steps = TextSteps {
            language: "fr".into(),
            ..all_on()
        };
        assert_eq!(process("J'ai vingt-cinq ans", &steps), "J'ai 25 ans");
        assert_eq!(process("quatre-vingt-dix-huit", &steps), "98");
        assert_eq!(process("trois pour cent point", &steps), "3 %.");
    }

    #[test]
    fn numbers_wait_for_french() {
        for language in ["", "en"] {
            let steps = TextSteps {
                language: language.into(),
                ..TextSteps::default()
            };
            assert_eq!(process("vingt-cinq", &steps), "vingt-cinq");
        }
    }
}
