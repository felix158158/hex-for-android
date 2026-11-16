//! Desktop check of the Android core: `cargo run --release --example smoke --
//! MODEL.gguf CLIP.wav [TERM[=SOUNDS,LIKE] ...]`. The clip must be 16 kHz mono PCM16.
//! Prints the raw transcript, the processed text, and the timing.

use hexcore::engine::{Engine, RunSettings, VocabularyTerm};
use hexcore::text::{TextSteps, process};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (model, clip, terms) = (&args[0], &args[1], &args[2..]);
    let pcm = read_wav(clip);
    let threads = std::env::var("HEX_THREADS").ok().and_then(|t| t.parse().ok()).unwrap_or(4);
    let engine = Engine::load(model, threads).expect("load");
    eprintln!("{}", engine.describe());
    let settings = RunSettings {
        vocabulary: terms
            .iter()
            .map(|t| {
                let (term, sounds) = t.split_once('=').unwrap_or((t, ""));
                VocabularyTerm {
                    term: term.to_owned(),
                    sounds_like: sounds.split(',').filter(|s| !s.is_empty()).map(str::to_owned).collect(),
                }
            })
            .collect(),
        boost: (!terms.is_empty()).then_some(2.5),
        // Parakeet v3 and the other multilingual models work the language out
        // themselves, so the hint stays empty unless HEX_LANG names one.
        language: std::env::var("HEX_LANG").ok().filter(|l| !l.is_empty()),
    };
    // The app keeps the model loaded, so the warm runs are the ones a
    // dictation sees. HEX_RUNS sets how many follow the cold one.
    let runs = std::env::var("HEX_RUNS").ok().and_then(|r| r.parse().ok()).unwrap_or(0);
    let mut run = engine.transcribe(&pcm, &settings).expect("transcribe");
    eprintln!("cold: {} ms", run.inference_ms);
    for _ in 0..runs {
        run = engine.transcribe(&pcm, &settings).expect("transcribe");
        eprintln!("warm: {} ms", run.inference_ms);
    }
    let steps = TextSteps {
        spoken_formatting: true,
        spoken_symbols: true,
        filler_words: true,
        language: std::env::var("HEX_LANG").unwrap_or_else(|_| "en".into()),
        replacements: vec![],
    };
    println!("raw:       {}", run.text);
    println!("processed: {}", process(&run.text, &steps));
    println!(
        "audio {} ms, inference {} ms, RTF {:.3}",
        run.audio_ms,
        run.inference_ms,
        run.inference_ms as f64 / run.audio_ms.max(1) as f64
    );
}

fn read_wav(path: &str) -> Vec<f32> {
    let bytes = std::fs::read(path).expect("read clip");
    let data = bytes.windows(4).position(|w| w == b"data").expect("data chunk") + 8;
    bytes[data..]
        .chunks_exact(2)
        .map(|s| i16::from_le_bytes([s[0], s[1]]) as f32 / 32768.0)
        .collect()
}
