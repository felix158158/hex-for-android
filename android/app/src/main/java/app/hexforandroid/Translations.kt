package app.hexforandroid

import java.util.Locale

/**
 * The interface's translations, following desktop Hex's own i18n tables in
 * `src/desktop/i18n/`: every string is keyed by its English source, and the
 * table is chosen from the system language. A string with no entry stays in
 * English, so a new language is a single additive table — see
 * `android/README.md#languages`.
 *
 * Translations are applied where the interface draws text (Ui.text and its
 * helpers, field hints, row descriptions, icon labels, toasts, sheet titles),
 * so no call site needs to know which language is active. Strings assembled
 * from several parts use the formatters below.
 */
object Translations {
    /** The active table's tag: "en" (no table), "fr", "zh", "sr", … */
    fun tag(): String {
        val locale = Locale.getDefault()
        return when {
            locale.language == "zh" && locale.script == "Hant" -> "zh-Hant"
            locale.language == "sr" && locale.script == "Latn" -> "sr-Latn"
            else -> locale.language
        }
    }

    private fun table(): Map<String, String>? = when (tag()) {
        "fr" -> French.strings
        "zh" -> Chinese.strings
        "sr" -> Serbian.strings
        else -> null
    }

    /** [value], translated when a table covers the phone's language. */
    fun of(value: CharSequence): CharSequence {
        val strings = table() ?: return value
        return strings[value.toString()] ?: value
    }

    /** [value] as a String, for places that build one from several parts. */
    fun s(value: String): String = of(value).toString()

    /** "3 terms", "1 term". */
    fun terms(count: Int): String = when (tag()) {
        "fr" -> if (count == 1) "1 terme" else "$count termes"
        "zh" -> "$count 个词条"
        "sr" -> if (count == 1) "1 термин" else "$count термина"
        else -> if (count == 1) "1 term" else "$count terms"
    }

    /** "2 replacements", "1 replacement". */
    fun replacements(count: Int): String = when (tag()) {
        "fr" -> if (count == 1) "1 remplacement" else "$count remplacements"
        "zh" -> "$count 条替换规则"
        "sr" -> if (count == 1) "1 замена" else "$count замене"
        else -> if (count == 1) "1 replacement" else "$count replacements"
    }

    /** "8.0 s of audio in 1.42 s". */
    fun dictation(audioMs: Long, inferenceMs: Long): String {
        val a = audioMs / 1000f
        val i = inferenceMs / 1000f
        return when (tag()) {
            "fr" -> "%.1f s d'audio en %.2f s".format(a, i)
            "zh" -> "%.1f 秒音频，耗时 %.2f 秒".format(a, i)
            "sr" -> "%.1f с звука за %.2f с".format(a, i)
            else -> "%.1f s of audio in %.2f s".format(a, i)
        }
    }

    /** "just now", "2 min ago", "3 h ago", "5 d ago". */
    fun ago(seconds: Long): String = when (tag()) {
        "fr" -> when {
            seconds < 45 -> "à l'instant"
            seconds < 3600 -> "il y a ${(seconds + 30) / 60} min"
            seconds < 86_400 -> "il y a ${seconds / 3600} h"
            else -> "il y a ${seconds / 86_400} j"
        }
        "zh" -> when {
            seconds < 45 -> "刚刚"
            seconds < 3600 -> "${(seconds + 30) / 60} 分钟前"
            seconds < 86_400 -> "${seconds / 3600} 小时前"
            else -> "${seconds / 86_400} 天前"
        }
        "sr" -> when {
            seconds < 45 -> "управо сада"
            seconds < 3600 -> "пре ${(seconds + 30) / 60} мин"
            seconds < 86_400 -> "пре ${seconds / 3600} ч"
            else -> "пре ${seconds / 86_400} дана"
        }
        else -> when {
            seconds < 45 -> "just now"
            seconds < 3600 -> "${(seconds + 30) / 60} min ago"
            seconds < 86_400 -> "${seconds / 3600} h ago"
            else -> "${seconds / 86_400} d ago"
        }
    }

    /** "Synced with the desktop 2 min ago". */
    fun syncedWith(name: String, ago: String): String = when (tag()) {
        "fr" -> "Synchronisé avec $name $ago"
        "zh" -> "已与 $name 同步，$ago"
        "sr" -> "Синхронизовано са $name $ago"
        else -> "Synced with $name $ago"
    }

    /** "Model: <detail>", the label translated, the detail left alone. */
    fun prefix(label: String, detail: String): String = when (tag()) {
        "zh" -> "${s(label)}：$detail"
        else -> "${s(label)} : $detail"
    }
}
