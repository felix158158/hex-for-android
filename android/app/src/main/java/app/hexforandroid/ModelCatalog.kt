package app.hexforandroid

/**
 * One GGUF the app can load: where to get it, how to check it, and what to
 * expect from it. Same fields as a desktop `GgufArtifact` in
 * `src/transcription/artifacts.rs`, pinned to the same revisions and hashes.
 *
 * @property takesLanguageHint false for models that detect the language by
 *   themselves (Parakeet v3), true for the ones desktop Hex sends a hint to
 *   (`accepts_language_hint` in models.rs).
 */
data class SpeechModel(
    val id: String,
    val label: String,
    val filename: String,
    val repository: String,
    val revision: String,
    val bytes: Long,
    val sha256: String,
    val takesLanguageHint: Boolean,
    val note: String,
)

/**
 * The models offered in Settings. Adding one is a new entry here: the store,
 * the picker and the setup step all read this list.
 */
object ModelCatalog {
    /** Parakeet TDT 0.6B v3: 25 European languages, detects the language itself. */
    val parakeetV3 = SpeechModel(
        id = "parakeet-v3",
        label = "Parakeet TDT 0.6B v3",
        filename = "parakeet-tdt-0.6b-v3-Q8_0.gguf",
        repository = "handy-computer/parakeet-tdt-0.6b-v3-gguf",
        revision = "85ac09ea12fc4b1112fa76810059364bc6adc9de",
        bytes = 739_508_576L,
        sha256 = "5859f77944efcd8eafa23a6350731960b2b55b2203df51f319665c807d802cc7",
        takesLanguageHint = false,
        note = "25 langues européennes, détecte la langue seule",
    )

    /** Whisper large-v3-turbo Q8_0: desktop Hex's multilingual model. */
    val whisperTurbo = SpeechModel(
        id = "whisper-turbo",
        label = "Whisper large-v3-turbo",
        filename = "whisper-large-v3-turbo-Q8_0.gguf",
        repository = "handy-computer/whisper-large-v3-turbo-gguf",
        revision = "d222c9f621c1128299248f2ded4d8a1820519780",
        bytes = 886_381_824L,
        sha256 = "d5e65f2b0828802ae2c231673d31982cebe3a778c95d9494a9f3efee6bd17448",
        takesLanguageHint = true,
        note = "environ 99 langues, plus lourd et plus lent",
    )

    /** Whisper large-v3 Q5_K_M: the full model, quantised to stay loadable. */
    val whisperLargeQ5 = SpeechModel(
        id = "whisper-large-q5",
        label = "Whisper large-v3 (Q5_K_M)",
        filename = "whisper-large-v3-Q5_K_M.gguf",
        repository = "handy-computer/whisper-large-v3-gguf",
        revision = "274ad1d1cf680b7ee53cc70f78461394cd3a344b",
        bytes = 1_161_143_008L,
        sha256 = "6053d0fd69a0fd48b8fea5ea7a52b9e0cde389343566fa30e453a1b2b258dc38",
        takesLanguageHint = true,
        note = "environ 99 langues, qualité maximale, lent",
    )

    /** Whisper large-v3 Q8_0: the heaviest option, for a phone with headroom. */
    val whisperLargeQ8 = SpeechModel(
        id = "whisper-large-q8",
        label = "Whisper large-v3 (Q8_0)",
        filename = "whisper-large-v3-Q8_0.gguf",
        repository = "handy-computer/whisper-large-v3-gguf",
        revision = "274ad1d1cf680b7ee53cc70f78461394cd3a344b",
        bytes = 1_668_741_440L,
        sha256 = "2fa1a5f179f8a511a53e2108db270aa4af3ce08cd976af4180e2854666bb4ba3",
        takesLanguageHint = true,
        note = "1,7 Go en mémoire, très lent sur téléphone",
    )

    val all: List<SpeechModel> = listOf(parakeetV3, whisperTurbo, whisperLargeQ5, whisperLargeQ8)

    val default: SpeechModel = parakeetV3

    fun byId(id: String?): SpeechModel = all.firstOrNull { it.id == id } ?: default

    /** "739 Mo", "1,7 Go": the size the file takes on the phone. */
    fun sizeLabel(bytes: Long): String =
        if (bytes >= 1_000_000_000) "%.1f Go".format(bytes / 1e9) else "${bytes / 1_000_000} Mo"
}
