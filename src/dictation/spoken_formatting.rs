//! Labs › Spoken formatting: whole spoken phrases ("new line", "line
//! break", "new paragraph", "open quote", "close quote", and "quote" …
//! "unquote" around words, #1167) become line breaks
//! and quotation marks, in English and Spanish ("nuevo párrafo", "nueva
//! línea", "abre comillas", …), and in French when the language is French
//! ("nouvelle ligne", "nouveau paragraphe", "ouvrez les guillemets", …).
//! Matching ignores case and the accents of both, and needs word boundaries,
//! so "newline" or "new liner" stay as they are.
//! The commas, periods, and spaces a model puts around a command go with it.

#[derive(Clone, Copy)]
enum Command {
    Break(&'static str),
    OpenQuote,
    CloseQuote,
}

const COMMANDS: &[(&[&str], Command)] = &[
    (&["new", "paragraph"], Command::Break("\n\n")),
    (&["new", "line"], Command::Break("\n")),
    (&["line", "break"], Command::Break("\n")),
    (&["open", "quote"], Command::OpenQuote),
    (&["close", "quote"], Command::CloseQuote),
    (&["nuevo", "parrafo"], Command::Break("\n\n")),
    (&["nueva", "linea"], Command::Break("\n")),
    (&["salto", "de", "linea"], Command::Break("\n")),
    (&["abrir", "comillas"], Command::OpenQuote),
    (&["abre", "comillas"], Command::OpenQuote),
    (&["cerrar", "comillas"], Command::CloseQuote),
    (&["cierra", "comillas"], Command::CloseQuote),
];

/// The French phrases, used only when the dictation is French: "point" is an
/// English word too, so these wait for the language the text steps are told.
const COMMANDS_FR: &[(&[&str], Command)] = &[
    (&["nouveau", "paragraphe"], Command::Break("\n\n")),
    (&["nouvelle", "ligne"], Command::Break("\n")),
    (&["saut", "de", "ligne"], Command::Break("\n")),
    (&["retour", "a", "la", "ligne"], Command::Break("\n")),
    (&["a", "la", "ligne"], Command::Break("\n")),
    (&["ouvrez", "les", "guillemets"], Command::OpenQuote),
    (&["ouvre", "les", "guillemets"], Command::OpenQuote),
    (&["fermez", "les", "guillemets"], Command::CloseQuote),
    (&["ferme", "les", "guillemets"], Command::CloseQuote),
];

/// No extra phrases, for every language but French.
const NO_COMMANDS: &[(&[&str], Command)] = &[];

/// `c` lowercased with accents dropped, as the phrase lists are written:
/// "Párrafo" reads as "parrafo", "dièse" as "diese".
pub(super) fn fold(c: char) -> char {
    let mut lower = c.to_lowercase();
    let lower = match (lower.next(), lower.next()) {
        (Some(single), None) => single,
        _ => c,
    };
    // French accents and the typographic apostrophe, so one phrase list
    // matches "guillemets" however the model spells it.
    match lower {
        'à' | 'â' => return 'a',
        'è' | 'ê' | 'ë' => return 'e',
        'î' | 'ï' => return 'i',
        'ô' => return 'o',
        'ù' | 'û' => return 'u',
        'ç' => return 'c',
        'ÿ' => return 'y',
        '’' => return '\'',
        _ => {}
    }
    match lower {
        'á' => 'a',
        'é' => 'e',
        'í' => 'i',
        'ó' => 'o',
        'ú' | 'ü' => 'u',
        other => other,
    }
}

/// `transcript` with its spoken formatting phrases applied.
pub fn apply(transcript: &str) -> String {
    apply_for(transcript, "en")
}

/// `transcript` with the spoken formatting phrases of `language` applied.
pub fn apply_for(transcript: &str, language: &str) -> String {
    let chars: Vec<char> = transcript.chars().collect();
    let mut out = String::with_capacity(transcript.len());
    let mut index = 0;
    // Where the "unquote" closing an open "quote" starts.
    let mut pending_unquote: Option<usize> = None;
    while index < chars.len() {
        let word_start = index == 0 || !chars[index - 1].is_alphanumeric();
        if pending_unquote == Some(index) {
            pending_unquote = None;
            let end = index + "unquote".len();
            index = emit(&mut out, Command::CloseQuote, &chars, end);
            continue;
        }
        if word_start && let Some((command, end)) = command_at(&chars, index, language) {
            index = emit(&mut out, command, &chars, end);
            continue;
        }
        if word_start
            && pending_unquote.is_none()
            && let Some(end) = word_at(&chars, index, "quote")
            && let Some(close) = unquote_after(&chars, end)
        {
            pending_unquote = Some(close);
            index = emit(&mut out, Command::OpenQuote, &chars, end);
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

/// The command whose words start at `start`, and where they end.
fn command_at(chars: &[char], start: usize, language: &str) -> Option<(Command, usize)> {
    let extra = if language.starts_with("fr") {
        COMMANDS_FR
    } else {
        NO_COMMANDS
    };
    [COMMANDS, extra]
        .into_iter()
        .flat_map(|table| table.iter())
        .find_map(|(words, command)| {
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
        word_end.then_some((*command, position))
    })
}

/// Where the whole word `word` starting at `start` ends, if it is there.
fn word_at(chars: &[char], start: usize, word: &str) -> Option<usize> {
    let mut position = start;
    for expected in word.chars() {
        match chars.get(position) {
            Some(&c) if fold(c) == expected => position += 1,
            _ => return None,
        }
    }
    chars
        .get(position)
        .is_none_or(|next| !next.is_alphanumeric())
        .then_some(position)
}

/// Where the first whole-word "unquote" after `from` starts, when some words
/// come between: "quote unquote experts" is a figure of speech, not a quote.
fn unquote_after(chars: &[char], from: usize) -> Option<usize> {
    let mut words_between = false;
    for index in from..chars.len() {
        let word_start = index == 0 || !chars[index - 1].is_alphanumeric();
        if word_start && word_at(chars, index, "unquote").is_some() {
            return words_between.then_some(index);
        }
        words_between |= chars[index].is_alphanumeric();
    }
    None
}

fn is_space(c: char) -> bool {
    c.is_whitespace() && c != '\n'
}

fn trim_end(out: &mut String, also: &[char]) {
    while let Some(last) = out.chars().next_back() {
        if is_space(last) || also.contains(&last) {
            out.pop();
        } else {
            break;
        }
    }
}

fn skip(chars: &[char], mut index: usize, also: &[char]) -> usize {
    while chars
        .get(index)
        .is_some_and(|&c| is_space(c) || also.contains(&c))
    {
        index += 1;
    }
    index
}

/// Writes `command` and returns where the text continues.
fn emit(out: &mut String, command: Command, chars: &[char], end: usize) -> usize {
    match command {
        Command::Break(breaks) => {
            trim_end(out, &[',', ';']);
            out.push_str(breaks);
            skip(chars, end, &[',', '.', ';', ':'])
        }
        Command::OpenQuote => {
            trim_end(out, &[]);
            if out
                .chars()
                .next_back()
                .is_some_and(|last| !matches!(last, '\n' | '(' | '[' | '"'))
            {
                out.push(' ');
            }
            out.push('"');
            skip(chars, end, &[',', '.'])
        }
        Command::CloseQuote => {
            trim_end(out, &[',']);
            out.push('"');
            let next = skip(chars, end, &[]);
            if chars.get(next).is_some_and(|c| c.is_alphanumeric()) {
                out.push(' ');
            }
            next
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{apply, apply_for};

    #[test]
    fn breaks_replace_their_phrases_and_the_punctuation_around_them() {
        assert_eq!(apply("Hello. New line. Next"), "Hello.\nNext");
        assert_eq!(apply("One. New paragraph. Two."), "One.\n\nTwo.");
        assert_eq!(apply("one, line break, two"), "one\ntwo");
        assert_eq!(apply("a NEW LINE new line b"), "a\n\nb");
    }

    #[test]
    fn quotes_become_quotation_marks() {
        assert_eq!(
            apply("He said open quote hi close quote."),
            "He said \"hi\"."
        );
        assert_eq!(
            apply("He said, Open quote, hello, close quote, and left."),
            "He said, \"hello\", and left."
        );
        assert_eq!(apply("open quote yes close quote then"), "\"yes\" then");
    }

    #[test]
    fn quote_unquote_becomes_quotation_marks() {
        assert_eq!(apply("He said quote hello unquote."), "He said \"hello\".");
        assert_eq!(
            apply("Quote, something, unquote, and more"),
            "\"something\", and more"
        );
        assert_eq!(
            apply("open quote a close quote and quote b unquote"),
            "\"a\" and \"b\""
        );
        for text in [
            "Can I get a quote for this",
            "quote unquote experts",
            "an unquote alone",
            "quotes and quoted text unquoted",
        ] {
            assert_eq!(apply(text), text);
        }
    }

    #[test]
    fn spanish_phrases_work_with_or_without_accents_and_in_any_case() {
        assert_eq!(apply("Hola. Nuevo párrafo. Adiós."), "Hola.\n\nAdiós.");
        assert_eq!(apply("Hola. nuevo parrafo. Adiós."), "Hola.\n\nAdiós.");
        assert_eq!(apply("uno, nueva línea, dos"), "uno\ndos");
        assert_eq!(apply("uno NUEVA LINEA dos"), "uno\ndos");
        assert_eq!(apply("uno salto de línea dos"), "uno\ndos");
        assert_eq!(
            apply("dijo abre comillas hola cierra comillas"),
            "dijo \"hola\""
        );
        assert_eq!(
            apply("Dijo Abrir Comillas sí Cerrar Comillas y se fue"),
            "Dijo \"sí\" y se fue"
        );
        assert_eq!(apply("Hola. New paragraph. Adiós."), "Hola.\n\nAdiós.");
    }

    #[test]
    fn french_phrases_work_when_the_language_is_french() {
        assert_eq!(
            apply_for("Bonjour. Nouvelle ligne. Salut.", "fr"),
            "Bonjour.\nSalut."
        );
        assert_eq!(
            apply_for("Un. Nouveau paragraphe. Deux.", "fr"),
            "Un.\n\nDeux."
        );
        assert_eq!(apply_for("un, saut de ligne, deux", "fr"), "un\ndeux");
        assert_eq!(apply_for("un, retour à la ligne, deux", "fr"), "un\ndeux");
        assert_eq!(apply_for("un À LA LIGNE deux", "fr"), "un\ndeux");
        assert_eq!(
            apply_for("dit ouvrez les guillemets salut fermez les guillemets", "fr"),
            "dit \"salut\""
        );
        assert_eq!(
            apply_for("Il a dit, ouvrez les guillemets, oui, fermez les guillemets.", "fr"),
            "Il a dit, \"oui\"."
        );
    }

    #[test]
    fn french_phrases_wait_for_a_french_language() {
        for text in [
            "Bonjour. Nouvelle ligne. Salut.",
            "Un. Nouveau paragraphe. Deux.",
            "dit ouvrez les guillemets salut fermez les guillemets",
        ] {
            assert_eq!(apply(text), text);
            assert_eq!(apply_for(text, "en"), text);
        }
    }

    #[test]
    fn other_words_are_left_alone() {
        for text in [
            "comillas",
            "una línea lineal",
            "dos parrafos nuevos",
            "nuevo parrafos",
            "The newline character",
            "a new liner",
            "renew line items",
            "line breaker",
            "open quotes",
            "Plain text, nothing to do.",
            "",
        ] {
            assert_eq!(apply(text), text);
        }
    }
}
