package app.hexforandroid

import android.content.Context
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest
import java.util.concurrent.atomic.AtomicBoolean

/**
 * The GGUF files [ModelCatalog] offers, stored in the app's external files
 * directory so a development machine can also `adb push` one there instead of
 * downloading it.
 *
 * Every file is pinned by repository, revision, byte size and SHA-256, as the
 * desktop catalogue does, so a download is checked before the engine sees it.
 * A model is a parameter here: nothing in this object knows which one the user
 * picked.
 */
object ModelStore {
    fun dir(context: Context): File =
        File(context.getExternalFilesDir(null) ?: context.filesDir, "models").apply { mkdirs() }

    fun file(context: Context, model: SpeechModel) = File(dir(context), model.filename)

    private fun marker(context: Context, model: SpeechModel) =
        File(dir(context), "${model.filename}.verified")

    /** True when the file is there at the right size, hash or no hash. */
    fun isPresent(context: Context, model: SpeechModel) =
        file(context, model).let { it.isFile && it.length() == model.bytes }

    private fun stamp(context: Context, model: SpeechModel): String {
        val file = file(context, model)
        return "${file.length()}:${file.lastModified()}:${model.sha256}"
    }

    /** True when the file is present and its hash was checked for this size and time. */
    fun isVerified(context: Context, model: SpeechModel): Boolean {
        if (!isPresent(context, model)) return false
        val marker = marker(context, model)
        return marker.isFile && marker.readText() == stamp(context, model)
    }

    /** Hashes the file (a few seconds) and records the result. */
    fun verify(context: Context, model: SpeechModel, progress: (Float) -> Unit = {}): Boolean {
        val file = file(context, model)
        if (!file.isFile || file.length() != model.bytes) return false
        val digest = MessageDigest.getInstance("SHA-256")
        val buffer = ByteArray(1 shl 20)
        var read = 0L
        FileInputStream(file).use { input ->
            while (true) {
                val n = input.read(buffer)
                if (n < 0) break
                digest.update(buffer, 0, n)
                read += n
                progress(read.toFloat() / model.bytes)
            }
        }
        val ok = digest.digest().joinToString("") { "%02x".format(it) } == model.sha256
        val marker = marker(context, model)
        if (ok) marker.writeText(stamp(context, model)) else marker.delete()
        return ok
    }

    private val busy = AtomicBoolean(false)

    /** Downloads with resume from a partial file, then verifies. Blocking; one at a time. */
    fun download(context: Context, model: SpeechModel, progress: (Long, Long) -> Unit): Boolean {
        if (!busy.compareAndSet(false, true)) throw IllegalStateException("A download is already running")
        try {
            return downloadOnce(context, model, progress)
        } finally {
            busy.set(false)
        }
    }

    private fun downloadOnce(context: Context, model: SpeechModel, progress: (Long, Long) -> Unit): Boolean {
        val part = File(dir(context), "${model.filename}.part")
        var have = if (part.isFile) part.length() else 0L
        if (have > model.bytes) {
            part.delete()
            have = 0
        }
        if (have < model.bytes) {
            val url = "https://huggingface.co/${model.repository}/resolve/${model.revision}/${model.filename}"
            val connection = (URL(url).openConnection() as HttpURLConnection).apply {
                instanceFollowRedirects = true
                connectTimeout = 20_000
                readTimeout = 30_000
                if (have > 0) setRequestProperty("Range", "bytes=$have-")
            }
            val code = connection.responseCode
            if (code == 200 && have > 0) have = 0 // The server ignored the range.
            if (code != 200 && code != 206) throw IllegalStateException("HTTP $code from Hugging Face")
            connection.inputStream.use { input ->
                FileOutputStream(part, have > 0).use { output ->
                    val buffer = ByteArray(1 shl 16)
                    var last = 0L
                    while (true) {
                        val n = input.read(buffer)
                        if (n < 0) break
                        output.write(buffer, 0, n)
                        have += n
                        if (have - last > (1 shl 20)) {
                            progress(have, model.bytes)
                            last = have
                        }
                    }
                }
            }
        }
        // A stream that ended early keeps its partial file for the next resume.
        if (have != model.bytes) {
            throw IllegalStateException("The download stopped at ${have shr 20} MB; try again to resume")
        }
        progress(model.bytes, model.bytes)
        val file = file(context, model)
        file.delete()
        if (!part.renameTo(file)) throw IllegalStateException("could not move the download into place")
        return verify(context, model)
    }

    /** Removes the file and its verification marker. */
    fun delete(context: Context, model: SpeechModel) {
        file(context, model).delete()
        marker(context, model).delete()
    }
}
