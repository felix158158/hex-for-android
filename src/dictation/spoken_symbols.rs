//! Labs › Spoken symbols: whole spoken words for punctuation and symbols
//! become the characters, joined to their neighbours as written text joins
//! them: "hashtag release" is "#release", "my underscore variable" is
//! "my_variable", and "done comma thanks" is "done, thanks". Spanish words
//! ("arroba", "guion bajo", "punto y coma", …) work too, and French words
//! ("virgule", "point", "arobase", …) when the dictation is French. Matching
//! ignores case and the accents of both, and needs word boundaries, so
//! "commas" or "hashtags" stay as they are. The commas, periods, and spaces a
//! model puts around a spoken symbol go with it.

use super::spoken_formatting::fold;

/// Which neighbour a symbol attaches to.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Join {
    /// The next word: "#release".
    Next,
    /// Both words: "my_variable".
    Both,
    /// The previous word, with one space after: "done, thanks".
    Previous,
}

const SYMBOLS: &[(&[&str], char, Join)] = &[
    (&["hashtag"], '#', Join::Next),
    (&["hash", "sign"], '#', Join::Next),
    (&["at", "sign"], '@', Join::Next),
    (&["underscore"], '_', Join::Both),
    (&["hyphen"], '-', Join::Both),
    (&["slash"], '/', Join::Both),
    (&["comma"], ',', Join::Previous),
    (&["semicolon"], ';', Join::Previous),
    (&["colon"], ':', Join::Previous),
    (&["arroba"], '@', Join::Next),
    (&["guion", "bajo"], '_', Join::Both),
    (&["guion"], '-', Join::Both),
    (&["punto", "y", "coma"], ';', Join::Previous),
    (&["coma"], ',', Join::Previous),
    (&["dos", "puntos"], ':', Join::Previous),
];

/// The French words, used only when the dictation is French: "point" is an
/// English word too, so these wait for the language the text steps are told.
/// Longest phrases come first, because the first match wins.
const SYMBOLS_FR: &[(&[&str], char, Join)] = &[
    (&["point", "d'interrogation"], '?', Join::Previous),
    (&["point", "d'exclamation"], '!', Join::Previous),
    (&["point", "virgule"], ';', Join::Previous),
    (&["deux", "points"], ':', Join::Previous),
    (&["barre", "oblique"], '/', Join::Both),
    (&["virgule"], ',', Join::Previous),
    (&["point"], '.', Join::Previous),
    (&["pourcent"], '%', Join::Previous),
    (&["arobase"], '@', Join::Next),
    (&["diese"], '#', Join::Next),
    (&["souligne"], '_', Join::Both),
    // "la variable, soulignée" agrees with the noun it follows.
    (&["soulignee"], '_', Join::Both),
    (&["tiret"], '-', Join::Both),
    (&["egale"], '=', Join::Both),
    (&["asterisque"], '*', Join::Both),
];

/// No extra words, for every language but French.
const NO_SYMBOLS: &[(&[&str], char, Join)] = &[];

/// `transcript` with its spoken symbols applied.
pub fn apply(transcript: &str) -> String {
    apply_for(transcript, "en")
}

/// `transcript` with the spoken symbols of `language` applied.
pub fn apply_for(transcript: &str, language: &str) -> String {
    let chars: Vec<char> = transcript.chars().collect();
    let mut out = String::with_capacity(transcript.len());
    // Where the last symbol written ends, so trimming never removes it.
    let mut floor = 0;
    let mut index = 0;
    while index < chars.len() {
        let word_start = index == 0 || !chars[index - 1].is_alphanumeric();
        if word_start && let Some((symbol, join, end)) = symbol_at(&chars, index, language) {
            index = emit(&mut out, &mut floor, symbol, join, &chars, end);
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

/// The symbol whose words start at `start`, and where they end.
fn symbol_at(chars: &[char], start: usize, language: &str) -> Option<(char, Join, usize)> {
    let extra = if language.starts_with("fr") {
        SYMBOLS_FR
    } else {
        NO_SYMBOLS
    };
    [SYMBOLS, extra]
        .into_iter()
        .flat_map(|table| table.iter())
        .find_map(|(words, symbol, join)| {
        let mut position = start;
        for (number, word) in words.iter().enumerate() {
            if number > 0 {
                let spaces = chars[position..]
                    .iter()
                    .take_while(|c| c.is_whitespace())
                    .count();
                if spaces == 0 {
                    return None;
                }
                position += spaces;
            }
            for expected in word.chars() {
                match chars.get(position) {
                    Some(&c) if fold(c) == expected => position += 1,
                    _ => return None,
                }
            }
        }
        let word_end = chars
            .get(position)
            .is_none_or(|next| !next.is_alphanumeric());
        word_end.then_some((*symbol, *join, position))
    })
}

fn is_space(c: char) -> bool {
    c.is_whitespace() && c != '\n'
}

/// Removes trailing spaces and model commas, never below `floor`.
fn trim_end(out: &mut String, floor: usize) {
    while out.len() > floor
        && let Some(last) = out.chars().next_back()
        && (is_space(last) || last == ',')
    {
        out.pop();
    }
}

fn skip(chars: &[char], mut index: usize) -> usize {
    while chars
        .get(index)
        .is_some_and(|&c| is_space(c) || matches!(c, ',' | '.'))
    {
        index += 1;
    }
    index
}

/// Writes `symbol` and returns where the text continues.
fn emit(
    out: &mut String,
    floor: &mut usize,
    symbol: char,
    join: Join,
    chars: &[char],
    end: usize,
) -> usize {
    if join != Join::Next {
        trim_end(out, *floor);
    }
    out.push(symbol);
    *floor = out.len();
    let next = skip(chars, end);
    if join == Join::Previous
        && chars.get(next).is_some_and(|&c| {
            !matches!(
                c,
                '\n' | ',' | '.' | ';' | ':' | '!' | '?' | ')' | ']' | '"'
            )
        })
    {
        out.push(' ');
    }
    next
}

#[cfg(test)]
mod tests {
    use super::{apply, apply_for};

    #[test]
    fn symbols_join_their_neighbours() {
        assert_eq!(apply("hashtag release"), "#release");
        assert_eq!(apply("my underscore variable"), "my_variable");
        assert_eq!(apply("done comma thanks"), "done, thanks");
        assert_eq!(apply("tag it hash sign release"), "tag it #release");
        assert_eq!(apply("mail me at sign home"), "mail me @home");
        assert_eq!(apply("well hyphen known"), "well-known");
        assert_eq!(apply("and slash or"), "and/or");
        assert_eq!(apply("one semicolon two"), "one; two");
        assert_eq!(apply("the list colon apples"), "the list: apples");
        assert_eq!(apply("Done, Comma, thanks."), "Done, thanks.");
        assert_eq!(apply("the list COLON."), "the list:");
        assert_eq!(apply("a comma comma b"), "a,, b");
    }

    #[test]
    fn spanish_words_become_symbols_with_or_without_accents() {
        assert_eq!(apply("escríbeme a arroba casa"), "escríbeme a @casa");
        assert_eq!(apply("mi guion bajo variable"), "mi_variable");
        assert_eq!(apply("mi Guión Bajo variable"), "mi_variable");
        assert_eq!(apply("bien guión conocido"), "bien-conocido");
        assert_eq!(apply("uno punto y coma dos"), "uno; dos");
        assert_eq!(apply("hola coma gracias"), "hola, gracias");
        assert_eq!(apply("la lista dos puntos manzanas"), "la lista: manzanas");
        assert_eq!(apply("hola comma gracias"), "hola, gracias");
    }

    #[test]
    fn french_words_become_symbols_when_the_language_is_french() {
        assert_eq!(apply_for("bonjour virgule ça va", "fr"), "bonjour, ça va");
        assert_eq!(apply_for("fini point", "fr"), "fini.");
        assert_eq!(apply_for("un point virgule deux", "fr"), "un; deux");
        assert_eq!(apply_for("la liste deux points pommes", "fr"), "la liste: pommes");
        assert_eq!(apply_for("vraiment point d'interrogation", "fr"), "vraiment?");
        assert_eq!(apply_for("attention point d’exclamation", "fr"), "attention!");
        assert_eq!(apply_for("écris arobase maison", "fr"), "écris @maison");
        assert_eq!(apply_for("tag dièse sortie", "fr"), "tag #sortie");
        assert_eq!(apply_for("mon souligné variable", "fr"), "mon_variable");
        assert_eq!(apply_for("la valeur soulignée tva", "fr"), "la valeur_tva");
        assert_eq!(apply_for("dix pourcent de plus", "fr"), "dix% de plus");
        assert_eq!(apply_for("a égale b", "fr"), "a=b");
        assert_eq!(apply_for("bien tiret connu", "fr"), "bien-connu");
        assert_eq!(apply_for("et barre oblique ou", "fr"), "et/ou");
    }

    #[test]
    fn french_words_wait_for_a_french_language() {
        for text in ["fini point", "bonjour virgule ça va", "tag dièse sortie"] {
            assert_eq!(apply(text), text);
            assert_eq!(apply_for(text, "en"), text);
        }
    }

    #[test]
    fn other_words_are_left_alone() {
        for text in [
            "comas",
            "guiones largos",
            "un punto y aparte",
            "commas",
            "slashed prices",
            "hashtags everywhere",
            "a hash browns sign",
            "the colonel",
            "Plain text, nothing to do.",
            "",
        ] {
            assert_eq!(apply(text), text);
        }
    }
}
