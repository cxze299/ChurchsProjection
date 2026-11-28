package org.churchpresenter.presentationengine

import org.churchpresenter.presentationengine.cache.SlideDiskCache
import java.io.File
import java.io.IOException
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit

/**
 * Converts PowerPoint decks (`.ppsx`, `.pps`, `.ppt`, `.pptx`, ...) to cached PDFs using an
 * installed native office suite (Microsoft PowerPoint / WPS Presentation / LibreOffice / Keynote)
 * so musical scores, embedded subset fonts, equations, tabs, and WMF/EMF metafiles render with
 * native layout fidelity. Returns `null` whenever no converter is available or conversion fails so
 * [DeckRasterizer] can fall back seamlessly to the in-JVM POI engine.
 */
internal object PowerPointDocumentPdf {
    private const val CONVERSION_TIMEOUT_SECONDS = 90L
    private const val EXTERNAL_PDF_PROPERTY = "churchpresenter.pptx.externalPdf"
    private val conversionLock = Any()
    private val failedIdentities = ConcurrentHashMap.newKeySet<String>()

    /** Test hook: overrides external conversion without invoking host Office/LibreOffice binaries. */
    @Volatile
    internal var converterOverride: ((source: File, workDir: File) -> File?)? = null

    /** Test hook: overrides the PDF cache root directory. */
    @Volatile
    internal var cacheBaseDir: File = File(System.getProperty("user.home"), ".churchpresenter/presentation-pdf")

    private val runningUnderUnitTest: Boolean by lazy {
        System.getProperty("java.class.path", "").contains("junit") ||
            Thread.currentThread().stackTrace.any { it.className.startsWith("org.junit.") }
    }

    fun isEnabled(): Boolean {
        val explicit = System.getProperty(EXTERNAL_PDF_PROPERTY)
        if (explicit != null) return explicit.equals("true", ignoreCase = true)
        if (converterOverride != null) return true
        return !runningUnderUnitTest
    }

    fun convertOrNull(file: File): File? {
        if (!isEnabled() || !file.isFile) return null
        val identity = fileIdentity(file)
        if (identity in failedIdentities) return null
        return synchronized(conversionLock) {
            val cached = cacheFile(file)
            if (cached.isFile && cached.length() > 0) return@synchronized cached
            if (identity in failedIdentities) return@synchronized null
            val cacheDir = cached.parentFile
            if (!cacheDir.isDirectory && !cacheDir.mkdirs()) {
                failedIdentities.add(identity)
                return@synchronized null
            }
            val work = try {
                Files.createTempDirectory(cacheDir.toPath(), "conversion-").toFile()
            } catch (_: IOException) {
                failedIdentities.add(identity)
                return@synchronized null
            }
            try {
                val converted = runCatching { render(file, work) }.getOrNull()
                if (converted == null || !converted.isFile || converted.length() == 0L) {
                    failedIdentities.add(identity)
                    return@synchronized null
                }
                Files.move(converted.toPath(), cached.toPath(), StandardCopyOption.REPLACE_EXISTING)
                cached
            } catch (_: Exception) {
                failedIdentities.add(identity)
                null
            } finally {
                work.deleteRecursively()
            }
        }
    }

    fun cacheFile(file: File): File {
        val key = "${SlideDiskCache.idFor(file.canonicalPath)}_${file.lastModified()}_${file.length()}"
        return File(File(cacheBaseDir, key), "deck.pdf")
    }

    internal fun clearFailureCache() {
        failedIdentities.clear()
    }

    private fun fileIdentity(file: File): String =
        "${file.canonicalPath}:${file.lastModified()}:${file.length()}"

    private fun render(file: File, work: File): File? {
        converterOverride?.let { return it(file, work) }
        val os = System.getProperty("os.name", "").lowercase()
        return when {
            os.contains("win") -> renderOnWindows(file, work)
            os.contains("mac") -> renderOnMac(file, work)
            else -> convertWithLibreOffice(file, work)
        }
    }

    private fun renderOnWindows(file: File, work: File): File? {
        // Prefer native PowerPoint / WPS Presentation on Windows for exact CJK score & font fidelity.
        convertWithWindowsCom(file, work)?.let { return it }
        return convertWithLibreOffice(file, work)
    }

    private fun renderOnMac(file: File, work: File): File? {
        // LibreOffice runs fully headless on macOS; fall back to Keynote AppleScript if installed.
        convertWithLibreOffice(file, work)?.let { return it }
        return convertWithMacKeynote(file, work)
    }

    private fun convertWithWindowsCom(file: File, work: File): File? {
        val output = File(work, "deck.pdf")
        val builder = ProcessBuilder("powershell.exe", "-NoProfile", "-NonInteractive", "-Command", PPT_EXPORT_SCRIPT)
        builder.environment()["CHURCHPRESENTER_PPT_SOURCE"] = file.absolutePath
        builder.environment()["CHURCHPRESENTER_PPT_PDF"] = output.absolutePath
        return if (runProcess(builder) && output.isFile && output.length() > 0) output else null
    }

    private fun convertWithLibreOffice(file: File, work: File): File? {
        val office = libreOfficeExecutable() ?: return null
        val profile = File(work, "profile").toURI().toString()
        val builder = ProcessBuilder(
            office,
            "-env:UserInstallation=$profile",
            "--headless",
            "--convert-to",
            "pdf",
            "--outdir",
            work.absolutePath,
            file.absolutePath,
        )
        val output = File(work, "${file.nameWithoutExtension}.pdf")
        return if (runProcess(builder) && output.isFile && output.length() > 0) output else null
    }

    private fun convertWithMacKeynote(file: File, work: File): File? {
        val keynoteApp = listOf(
            File("/Applications/Keynote.app"),
            File("${System.getProperty("user.home")}/Applications/Keynote.app"),
        ).firstOrNull { it.isDirectory } ?: return null
        val output = File(work, "deck.pdf")
        val builder = ProcessBuilder(
            "osascript",
            "-e",
            MAC_KEYNOTE_EXPORT_SCRIPT,
            keynoteApp.absolutePath,
            file.absolutePath,
            output.absolutePath,
        )
        return if (runProcess(builder) && output.isFile && output.length() > 0) output else null
    }

    private fun libreOfficeExecutable(): String? {
        val home = System.getProperty("user.home")
        val candidates = listOf(
            "C:/Program Files/LibreOffice/program/soffice.com",
            "C:/Program Files/LibreOffice/program/soffice.exe",
            "C:/Program Files (x86)/LibreOffice/program/soffice.com",
            "C:/Program Files (x86)/LibreOffice/program/soffice.exe",
            "/Applications/LibreOffice.app/Contents/MacOS/soffice",
            "$home/Applications/LibreOffice.app/Contents/MacOS/soffice",
            "/opt/homebrew/bin/soffice",
            "/usr/local/bin/soffice",
            "/usr/bin/soffice",
        ) + System.getenv("PATH").orEmpty().split(File.pathSeparator).flatMap {
            listOf(File(it, "soffice").path, File(it, "soffice.com").path, File(it, "soffice.exe").path)
        }
        return candidates.firstOrNull { File(it).isFile }
    }

    private fun runProcess(builder: ProcessBuilder): Boolean {
        val process = try {
            builder.redirectOutput(ProcessBuilder.Redirect.DISCARD)
                .redirectError(ProcessBuilder.Redirect.DISCARD)
                .start()
        } catch (_: IOException) {
            return false
        }
        return try {
            process.waitFor(CONVERSION_TIMEOUT_SECONDS, TimeUnit.SECONDS) && process.exitValue() == 0
        } finally {
            if (process.isAlive) process.destroyForcibly()
        }
    }

    private val PPT_EXPORT_SCRIPT = """
        ${'$'}ErrorActionPreference = 'Stop'
        ${'$'}office = ${'$'}null
        ${'$'}presentation = ${'$'}null
        try {
            try { ${'$'}office = New-Object -ComObject PowerPoint.Application }
            catch { ${'$'}office = New-Object -ComObject KWPP.Application }
            try { ${'$'}office.DisplayAlerts = 1 } catch { }
            ${'$'}presentation = ${'$'}office.Presentations.Open(${'$'}env:CHURCHPRESENTER_PPT_SOURCE, -1, 0, 0)
            ${'$'}presentation.ExportAsFixedFormat(${'$'}env:CHURCHPRESENTER_PPT_PDF, 2)
        } finally {
            if (${'$'}null -ne ${'$'}presentation) { try { ${'$'}presentation.Close() } catch { } }
            if (${'$'}null -ne ${'$'}office) { try { ${'$'}office.Quit() } catch { } }
        }
    """.trimIndent()

    private val MAC_KEYNOTE_EXPORT_SCRIPT = """
        on run argv
            set srcPath to item 2 of argv
            set dstPath to item 3 of argv
            tell application "Keynote"
                set doc to open (POSIX file srcPath)
                export doc to (POSIX file dstPath) as PDF
                close doc saving no
            end tell
        end run
    """.trimIndent()
}
