package app.hexforandroid

import java.util.Locale

/**
 * The French interface, keyed by the English source string and applied where
 * the interface draws text, so a missing entry falls back to English rather
 * than leaving a blank row.
 *
 * The wording follows desktop Hex's French table
 * (`src/desktop/i18n/french.rs`), which is what a French system language
 * picks up on the desktop: "dictée", "raccourci", "vocabulaire", "Réglages".
 * Anything the desktop does not translate yet is written here in the same
 * register, tutoiement included, as the rest of the app's copy.
 */
object Text {
    private fun french() = Locale.getDefault().language == "fr"

    /** [value], translated when the phone is in French. */
    fun of(value: CharSequence): CharSequence {
        if (!french()) return value
        return FRENCH[value.toString()] ?: value
    }

    /** [value] as a String, for places that build one from several parts. */
    fun s(value: String): String = of(value).toString()

    /** "3 terms", "1 terme". */
    fun terms(count: Int): String = when {
        !french() -> if (count == 1) "1 term" else "$count terms"
        count == 1 -> "1 terme"
        else -> "$count termes"
    }

    /** "2 replacements", "2 remplacements". */
    fun replacements(count: Int): String = when {
        !french() -> if (count == 1) "1 replacement" else "$count replacements"
        count == 1 -> "1 remplacement"
        else -> "$count remplacements"
    }

    /** "8.0 s of audio in 1.42 s". */
    fun dictation(audioMs: Long, inferenceMs: Long): String = if (french()) {
        "%.1f s d'audio en %.2f s".format(audioMs / 1000f, inferenceMs / 1000f)
    } else {
        "%.1f s of audio in %.2f s".format(audioMs / 1000f, inferenceMs / 1000f)
    }

    /** "just now", "2 min ago", "3 h ago", "5 d ago". */
    fun ago(seconds: Long): String = when {
        !french() -> when {
            seconds < 45 -> "just now"
            seconds < 3600 -> "${(seconds + 30) / 60} min ago"
            seconds < 86_400 -> "${seconds / 3600} h ago"
            else -> "${seconds / 86_400} d ago"
        }
        seconds < 45 -> "à l'instant"
        seconds < 3600 -> "il y a ${(seconds + 30) / 60} min"
        seconds < 86_400 -> "il y a ${seconds / 3600} h"
        else -> "il y a ${seconds / 86_400} j"
    }

    /** "Synced with the desktop 2 min ago". */
    fun syncedWith(name: String, ago: String): String =
        if (french()) "Synchronisé avec $name $ago" else "Synced with $name $ago"

    /** "Model: <detail>", the label translated and the detail left alone. */
    fun prefix(label: String, detail: String): String = "${s(label)} : $detail"

    @Suppress("SpellCheckingInspection")
    private val FRENCH: Map<String, String> = mapOf(
        // Sections and the set-up list.
        "Set up" to "Configurer",
        "Microphone" to "Micro",
        "Notifications" to "Notifications",
        "Accessibility" to "Accessibilité",
        "Battery" to "Batterie",
        "Speech model" to "Modèle vocal",
        "Bubble" to "Bulle",
        "Dictation" to "Dictée",
        "Advanced" to "Avancé",
        "About" to "À propos",

        // Rows.
        "Shape" to "Forme",
        "Size" to "Taille",
        "Show" to "Afficher",
        "Clean-up" to "Nettoyage",
        "Vocabulary" to "Vocabulaire",
        "Practice" to "Essai",
        "History" to "Historique",
        "Sounds" to "Sons",
        "Desktop sync" to "Synchro bureau",
        "Insert text" to "Insérer le texte",
        "Lower other audio" to "Baisser les autres sons",
        "CPU threads" to "Threads CPU",
        "Spoken formatting" to "Mise en forme vocale",
        "Formatting" to "Mise en forme",
        "Symbols" to "Symboles",
        "Fillers" to "Hésitations",
        "Model" to "Modèle",
        "Spoken symbols" to "Symboles vocaux",
        "Remove filler words" to "Supprimer les mots de remplissage",
        "Replace words" to "Remplacer des mots",
        "Snap to vocabulary" to "Aligner sur le vocabulaire",
        "Strength" to "Force",
        "File" to "Fichier",
        "Engine" to "Moteur",
        "Last dictation" to "Dernière dictée",

        // Choices.
        "Off" to "Désactivé",
        "On" to "Activé",
        "None" to "Aucun",
        "Light" to "Léger",
        "Normal" to "Normal",
        "Strong" to "Fort",
        "Small" to "Petite",
        "Medium" to "Moyenne",
        "Large" to "Grande",
        "Circle" to "Cercle",
        "Square" to "Carré",
        "Hexagon" to "Hexagone",
        "Pill" to "Pilule",
        "While typing" to "Pendant la saisie",

        // Buttons and actions.
        "Done" to "Terminé",
        "Cancel" to "Annuler",
        "Add" to "Ajouter",
        "Add term" to "Ajouter un terme",
        "Add rule" to "Ajouter une règle",
        "Clear" to "Effacer",
        "Clear." to "Effacer.",
        "Clear history?" to "Effacer l'historique ?",
        "Copy" to "Copier",
        "Copied" to "Copié",
        "Download" to "Télécharger",
        "Verify" to "Vérifier",
        "Unpair" to "Dissocier",
        "Sync now" to "Synchroniser",
        "Paste last" to "Coller la dernière",
        "Turn on" to "Activer",
        "Turn off" to "Désactiver",
        "Turn Hex on first" to "Active Hex d'abord",
        "App info" to "Infos de l'app",
        "Allow" to "Autoriser",
        "Open" to "Ouvrir",
        "Try" to "Essayer",
        "Preview" to "Aperçu",
        "Switch" to "Interrupteur",
        "Dictate" to "Dicter",
        "Show all" to "Tout afficher",
        "Nothing yet" to "Rien pour l'instant",

        // Dictation states.
        "Listening." to "J'écoute.",
        "Listening" to "J'écoute",
        "Starting" to "Démarrage",
        "Transcribing." to "Transcription.",
        "Transcribing" to "Transcription",
        "Loading the speech model…" to "Chargement du modèle vocal…",
        "Nothing heard" to "Rien entendu",
        "Loads when Hex turns on" to "Chargé à l'activation de Hex",
        "Not downloaded" to "Non téléchargé",
        "not downloaded" to "non téléchargé",
        "verified" to "vérifié",
        "not verified" to "non vérifié",
        "Hex is on" to "Hex est actif",
        "Turn on to show the bubble" to "Active pour afficher la bulle",
        "Parakeet v3 (multilingual)" to "Parakeet v3 (multilingue)",
        "Parakeet v3 · 739 MB" to "Parakeet v3 · 739 Mo",
        "739 MB · verified" to "739 Mo · vérifié",
        "739 MB · not verified" to "739 Mo · non vérifié",

        // Descriptions.
        "Shows when Hex is on" to "S'affiche quand Hex est actif",
        "Shows the bubble and types for you" to "Affiche la bulle et tape à ta place",
        "Records only while you dictate" to "N'enregistre que pendant la dictée",
        "Keeps Hex ready in the background" to "Garde Hex prêt en arrière-plan",
        "Paste uses the clipboard" to "Coller passe par le presse-papiers",
        "New line, open quote, close quote" to "Nouvelle ligne, guillemets ouverts, guillemets fermés",
        "Hashtag, underscore, at sign" to "Dièse, souligné, arobase",
        "Um, uh, er" to "Euh, hum",
        "Runs top to bottom" to "De haut en bas",
        "Runs on every dictation, top to bottom" to "S'applique à chaque dictée, de haut en bas",
        "Matches ignore case. An empty output removes the phrase" to "La casse est ignorée. Une sortie vide supprime la phrase",
        "Under Sounds like, add what the model writes instead" to "Sous « Sonne comme », indique ce que le modèle écrit à la place",
        "Sounds like" to "Sonne comme",
        "Term | sounds like" to "Terme | sonne comme",
        "Bias Parakeet toward these terms" to "Oriente Parakeet vers ces termes",
        "Try the bubble without leaving Hex" to "Essaie la bulle sans quitter Hex",
        "Bubble preview" to "Aperçu de la bulle",
        "Shown while Hex is ready to dictate" to "Visible quand Hex est prêt à dicter",
        "Tap the field, then use the bubble, or Dictate" to "Appuie dans le champ, puis utilise la bulle, ou Dicter",
        "Tap the bubble, or hold it to talk" to "Appuie sur la bulle, ou maintiens pour parler",
        "Tap the bubble to stop" to "Appuie sur la bulle pour arrêter",
        "Hex keeps your last 50 dictations on this phone" to "Hex garde tes 50 dernières dictées sur ce téléphone",
        "Dictated text appears here" to "Le texte dicté apparaît ici",
        "Your dictations show here" to "Tes dictées apparaîtront ici",
        "Tap one to copy it" to "Appuie sur une ligne pour la copier",
        "just now" to "à l'instant",

        // Model sheet.
        "Parakeet v3, NVIDIA's multilingual model: 25 European languages, French among them. It picks the language you speak by itself, on this phone" to
            "Parakeet v3, le modèle multilingue de NVIDIA : 25 langues européennes, dont le français. Il devine la langue que tu parles, sur ce téléphone",
        "A download is already running" to "Un téléchargement est déjà en cours",
        "The model's checksum is wrong. Download it again" to "La somme de contrôle du modèle est fausse. Retélécharge-le",
        "The model file is damaged. Download it again" to "Le fichier du modèle est abîmé. Retélécharge-le",
        "The speech model isn't loaded" to "Le modèle vocal n'est pas chargé",
        "Download the speech model first" to "Télécharge d'abord le modèle vocal",

        // Set-up notes and failures.
        "Finish setting up first" to "Termine d'abord la configuration",
        "Finish setting up below" to "Termine la configuration ci-dessous",
        "Applies when Hex next turns on" to "S'applique au prochain démarrage de Hex",
        "Hex needs the microphone permission" to "Hex a besoin de la permission micro",
        "Android blocked the microphone. Open Hex and turn it on again" to "Android a bloqué le micro. Ouvre Hex et réactive-le",
        "Android didn't let Hex start. Try again" to "Android n'a pas laissé Hex démarrer. Réessaie",
        "The microphone didn't open" to "Le micro n'a pas pu s'ouvrir",
        "The microphone stopped working" to "Le micro s'est arrêté",
        "Hex couldn't find the text field. It's on the clipboard" to "Hex n'a pas trouvé le champ. Le texte est dans le presse-papiers",
        "This field didn't accept the text. It's on the clipboard" to "Ce champ a refusé le texte. Il est dans le presse-papiers",
        "Copied. Turn on Hex's accessibility service to paste into apps" to "Copié. Active le service d'accessibilité de Hex pour coller dans les apps",

        // Desktop sync.
        "Not paired with desktop Hex" to "Non associé au Hex de bureau",
        "Sync failed" to "Échec de la synchro",
        "Not synced yet" to "Pas encore synchronisé",
        "Syncing." to "Synchronisation.",
        "Pair again: run hex-vocab-sync pair on the desktop" to "Réassocie : lance hex-vocab-sync pair sur le bureau",
        "Optional: keeps this vocabulary in step with desktop Hex on a Linux computer, over Tailscale" to
            "Optionnel : garde ce vocabulaire aligné avec Hex bureau sur un ordinateur Linux, via Tailscale",
        "To set it up, connect this phone to the computer by USB and run hex-vocab-sync pair there. The project's README has the steps" to
            "Pour l'activer, branche ce téléphone à l'ordinateur en USB et lance hex-vocab-sync pair dessus. Les étapes sont dans le README du projet",

        // Sounds.
        "Desktop Hex's sounds for start, stop and cancel. Tap a set to hear it" to
            "Les sons de Hex bureau pour démarrer, arrêter et annuler. Appuie sur un jeu pour l'entendre",
        "Soft tone" to "Tonalité douce",
        "Warm synth" to "Synthé chaleureux",
        "Hex signature" to "Signature Hex",
        "Mechanical key clicks" to "Clics mécaniques",
        "A mellow analogue synth" to "Un synthé analogique feutré",
        "A deep, creamy keyboard key" to "Une touche de clavier profonde",
        "Soft marimba and hollow wood blocks" to "Marimba doux et bois creux",
        "A kalimba motif that rises and comes home" to "Un motif de kalimba qui monte et revient",
        "Warm, rounded blips" to "Bips chauds et arrondis",
        "The tones Hex first shipped with" to "Les sons des débuts de Hex",

        // About.
        "An unofficial Android port of Hex. Speech is transcribed on this phone and never leaves it" to
            "Un portage Android non officiel de Hex. La parole est transcrite sur ce téléphone et n'en sort jamais",
        "Speech engine · MIT License" to "Moteur de parole · licence MIT",
        "Anomaly · MIT License" to "Anomaly · licence MIT",
    )
}
