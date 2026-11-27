package org.churchpresenter.presentationengine.fonts

import org.apache.poi.common.usermodel.fonts.FontInfo
import org.apache.poi.sl.draw.DrawFontManager
import org.apache.poi.sl.draw.DrawFontManagerDefault
import java.awt.Font
import java.awt.Graphics2D
import java.awt.GraphicsEnvironment
import java.io.File
import java.io.InputStream
import java.util.Locale
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Font resolution for slide rendering. Fixes the biggest static-fidelity gap of the old
 * pipeline: POI resolved fonts through plain `java.awt` with zero registration, so any family
 * not already known to the JVM was silently substituted with the default typeface.
 *
 * Three mechanisms, in order:
 *  1. [registerFontStream] — bundled app fonts are registered into the JVM's
 *     [GraphicsEnvironment] at startup so `new Font(family, …)` resolves them.
 *  2. [initialize] — platform font directories are scanned and any font file whose family the
 *     JVM does not already know is registered too. This covers per-user installed fonts
 *     (e.g. `%LOCALAPPDATA%\Microsoft\Windows\Fonts`, `~/.fonts`) that some JVMs miss.
 *  3. [drawFontManager] — installed as POI's `Drawable.FONT_HANDLER`: when a slide asks for a
 *     family that is not available even after 1+2, a curated substitution table picks the
 *     closest installed family (metric-compatible first, bundled app font as the safety net)
 *     instead of letting AWT fall back to Dialog.
 */
object SlideFontRegistry {

    /** Font folders nest a couple of levels (family/style); deeper is someone else's data. */
    private const val FONT_SCAN_DEPTH = 3

    /** Lower-cased family name → exact-cased family name, for every family the JVM can resolve. */
    private val availableFamilies = ConcurrentHashMap<String, String>()
    private val initialFamilyIndex = lazy { refreshAvailableFamilies() }

    /** Normalized file name (sans extension) → font file, from the system-directory scan. */
    private val systemFontFileIndex =
        FontFileIndex({ systemFontDirs }, FONT_SCAN_DEPTH) { normalizeName(it.nameWithoutExtension) }

    /**
     * Office-font substitution preferences. First installed candidate wins; every list ends in a
     * family bundled with the app (registered via [registerFontStream]) so slides never fall back
     * to the JVM's default Dialog face.
     */
    private val songti = listOf("SimSun", "Songti SC", "STSong", "Noto Serif CJK SC")
    private val heiti = listOf("SimHei", "Heiti SC", "PingFang SC", "Noto Sans CJK SC")
    private val kaiti = listOf("KaiTi", "STKaiti", "Kaiti SC", "Songti SC")
    private val fangsong = listOf("FangSong", "STFangsong", "Songti SC", "Noto Serif CJK SC")
    private val yahei = listOf("Microsoft YaHei", "PingFang SC", "Heiti SC", "Noto Sans CJK SC")
    private val jhenghei = listOf("Microsoft JhengHei", "PingFang TC", "Heiti TC", "Noto Sans CJK TC")
    private val mingliu = listOf("PMingLiU", "MingLiU", "Songti TC", "Noto Serif CJK TC")
    private val xingkai = listOf("STXingkai", "Xingkai SC", "STKaiti", "Kaiti SC")
    private val lishu = listOf("LiSu", "Baoli SC", "Songti SC")
    private val youyuan = listOf("YouYuan", "Yuanti SC", "PingFang SC")
    private val fontLocales = listOf(
        Locale.getDefault(), Locale.ROOT, Locale.ENGLISH, Locale.SIMPLIFIED_CHINESE, Locale.TRADITIONAL_CHINESE,
    ).distinct()

    private val substitutionTable: Map<String, List<String>> = mapOf(
        "calibri" to listOf("Carlito", "Segoe UI", "Open Sans", "Arial"),
        "calibri light" to listOf("Carlito", "Segoe UI Light", "Open Sans", "Arial"),
        "cambria" to listOf("Caladea", "Georgia", "Lora", "Times New Roman"),
        "segoe ui" to listOf("Open Sans", "Arial", "Helvetica Neue"),
        "arial" to listOf("Helvetica", "Liberation Sans", "Open Sans"),
        "helvetica" to listOf("Arial", "Liberation Sans", "Open Sans"),
        "helvetica neue" to listOf("Helvetica", "Arial", "Open Sans"),
        "times new roman" to listOf("Liberation Serif", "Times", "Georgia", "Lora"),
        "georgia" to listOf("Times New Roman", "Lora"),
        "courier new" to listOf("Liberation Mono", "Courier", "Anonymous Pro"),
        "consolas" to listOf("Menlo", "Liberation Mono", "Anonymous Pro"),
        "comic sans ms" to listOf("Kalam", "Open Sans"),
        "tahoma" to listOf("Verdana", "DejaVu Sans", "Open Sans"),
        "verdana" to listOf("Tahoma", "DejaVu Sans", "Open Sans"),
        "trebuchet ms" to listOf("Verdana", "Open Sans"),
        "century gothic" to listOf("Avant Garde", "Poppins", "Open Sans"),
        "gill sans" to listOf("Gill Sans MT", "Open Sans"),
        "impact" to listOf("Haettenschweiler", "Anton", "Open Sans"),
        "simsun" to songti,
        "nsimsun" to songti,
        "宋体" to songti,
        "新宋体" to songti,
        "simhei" to heiti,
        "黑体" to heiti,
        "kaiti" to kaiti,
        "kaiti_gb2312" to kaiti,
        "楷体" to kaiti,
        "楷体_gb2312" to kaiti,
        "fangsong" to fangsong,
        "fangsong_gb2312" to fangsong,
        "仿宋" to fangsong,
        "仿宋_gb2312" to fangsong,
        "microsoft yahei" to yahei,
        "微软雅黑" to yahei,
        "微軟雅黑" to yahei,
        "dengxian" to yahei,
        "等线" to yahei,
        "等線" to yahei,
        "stkaiti" to kaiti,
        "华文楷体" to kaiti,
        "華文楷體" to kaiti,
        "dfkai-sb" to listOf("DFKai-SB", "Kaiti TC", "STKaiti", "Songti TC"),
        "標楷體" to listOf("DFKai-SB", "Kaiti TC", "STKaiti", "Songti TC"),
        "stxingkai" to xingkai,
        "华文行楷" to xingkai,
        "華文行楷" to xingkai,
        "lisu" to lishu,
        "隶书" to lishu,
        "隸書" to lishu,
        "youyuan" to youyuan,
        "幼圆" to youyuan,
        "幼圓" to youyuan,
        "microsoft jhenghei" to jhenghei,
        "微軟正黑體" to jhenghei,
        "pmingliu" to mingliu,
        "mingliu" to mingliu,
        "新細明體" to mingliu,
        "細明體" to mingliu,
    )

    /**
     * Fonts the engine carries itself (`resources/slide-fonts`, SIL Open Font License): Carlito,
     * drawn to Calibri's metrics. Calibri is the body font of most decks and comes only with
     * Office, so without it a Calibri slide is set in a wider face and its lines re-wrap and
     * overflow; with it every line breaks where PowerPoint breaks it.
     */
    private val BUNDLED_FONTS = listOf(
        "Carlito-Regular.ttf", "Carlito-Bold.ttf", "Carlito-Italic.ttf", "Carlito-BoldItalic.ttf",
    )

    private val bundledFontsRegistered = AtomicBoolean(false)

    /** The ultimate fallback family when nothing else matches (bundled with the app). */
    private const val DEFAULT_FAMILY = "Open Sans"

    /** A requested typeface resolved to a renderable family, with any style its name carried. */
    data class ResolvedFace(val family: String, val bold: Boolean, val italic: Boolean)

    private val systemFontDirs: List<File> by lazy {
        val os = System.getProperty("os.name", "").lowercase()
        val home = System.getProperty("user.home")
        when {
            os.contains("mac") -> listOf(
                "/System/Library/Fonts",
                "/System/Library/Fonts/Supplemental",
                "/Library/Fonts",
                "$home/Library/Fonts",
                "$home/Library/Group Containers/UBF8T346G9.Office/FontCache",
            ) + listOf("/Applications", "$home/Applications").flatMap { applications ->
                listOf("PowerPoint", "Word", "Excel").map { "$applications/Microsoft $it.app/Contents/Resources/DFonts" } +
                    listOf("$applications/wpsoffice.app/Contents/Resources/fonts")
            } + File("/System/Library/AssetsV2").listFiles().orEmpty()
                .filter { it.isDirectory && it.name.startsWith("com_apple_MobileAsset_Font") }
                .flatMap { asset -> asset.listFiles().orEmpty().map { File(it, "AssetData").path } }
            os.contains("win") -> listOf(
                (System.getenv("WINDIR") ?: "C:\\Windows") + "\\Fonts",
                "$home\\AppData\\Local\\Microsoft\\Windows\\Fonts"
            )
            else -> listOf(
                "/usr/share/fonts",
                "/usr/local/share/fonts",
                "$home/.fonts",
                "$home/.local/share/fonts"
            )
        }.map(::File)
    }

    /** POI font handler — set as the `Drawable.FONT_HANDLER` rendering hint before slide draws. */
    val drawFontManager: DrawFontManager = SubstitutingFontManager()

    /**
     * Registers a TTF/OTF font or collection into the JVM. Returns the first family on success.
     * Safe to call repeatedly; registration of an already-known family is a no-op in AWT.
     */
    fun registerFontStream(stream: InputStream): String? {
        return try {
            val fonts = stream.use { Font.createFonts(it) }
            val graphics = GraphicsEnvironment.getLocalGraphicsEnvironment()
            fonts.forEach { font ->
                graphics.registerFont(font)
                indexFamily(font)
            }
            fonts.firstOrNull()?.family
        } catch (_: Exception) {
            null
        }
    }

    /**
     * Builds the family index and (optionally) scans platform font directories, registering any
     * font file whose family the JVM does not already resolve. Idempotent; call once at startup
     * on a background thread — a directory scan can take a couple of seconds on font-heavy
     * systems.
     */
    fun initialize(scanSystemDirs: Boolean = true) {
        initialFamilyIndex.value
        if (scanSystemDirs) systemFontRegistration.value
    }

    private val systemFontRegistration = lazy {
        // One walk completes the file index and registers the families; only when a lookup has
        // already completed the index does this walk the folders again, to register.
        if (!systemFontFileIndex.scanAll(::registerFileIfUnknown)) {
            for (dir in systemFontDirs) {
                if (!dir.isDirectory) continue
                dir.walkTopDown().maxDepth(FONT_SCAN_DEPTH)
                    .filter { it.isFile && it.extension.lowercase() in FontFileIndex.FONT_FILE_EXTENSIONS }
                    .forEach(::registerFileIfUnknown)
            }
        }
    }

    fun isFamilyAvailable(family: String): Boolean {
        initialFamilyIndex.value
        return availableFamilies.containsKey(cleanTypeface(family).lowercase())
    }

    /**
     * Resolves the family a slide should actually render with: the requested family when
     * installed, else the first installed substitution candidate, else [DEFAULT_FAMILY] when
     * available, else the JVM's logical SansSerif.
     */
    fun resolveFamily(requested: String): String = resolveFace(requested).family

    /**
     * Resolves a requested typeface to a renderable family, plus the weight and slant the request
     * only carried in its *name*.
     *
     * Keynote stores the typeface as a **PostScript name** (`Arial-BoldMT`, `HelveticaNeue-Bold`,
     * `TimesNewRomanPS-BoldMT`), never as the AWT family name, so a plain family lookup misses on
     * essentially every real deck and everything used to land on [DEFAULT_FAMILY]. Since the
     * substitute is a different width, that silently re-wrapped slides: `Arial-BoldMT` at 38pt
     * measured 922pt as Arial and 987pt as Open Sans, which is one extra line in a 952.8pt box.
     *
     * [PostScriptName] therefore peels the name back to its family a step at a time, and each
     * candidate is tried against the installed families *and* [substitutionTable], so
     * `HelveticaNeue-Bold` still reaches the "helvetica neue" row on a machine without it.
     */
    fun resolveFace(typeface: String): ResolvedFace {
        // Embedded fonts can be registered before the installed families have been loaded.
        initialFamilyIndex.value
        val requested = cleanTypeface(typeface)
        val (bold, italic) = PostScriptName.styleOf(requested)
        // An installed face first when the name carries a width or weight a family cannot (the
        // face already has the weight and slant its name says; reporting them as well costs
        // nothing, since asking a face for a style it has adds nothing to it). Then the name
        // peeled back to a family, installed or substituted, then the default.
        val face = if (PostScriptName.namesFaceOnlyStyle(requested)) InstalledFaces.find(requested) else null
        val family = face
            ?: PostScriptName.candidates(requested).firstNotNullOfOrNull { candidate ->
                val key = candidate.lowercase()
                availableFamilies[key]
                    ?: substitutionTable[key]?.firstNotNullOfOrNull { availableFamilies[it.lowercase()] }
            }
            ?: availableFamilies[DEFAULT_FAMILY.lowercase()]
            ?: Font.SANS_SERIF
        return ResolvedFace(family, bold, italic)
    }

    /**
     * Finds an installed font *file* by common naming patterns — used by the app's Lottie font
     * loader, which needs raw bytes rather than an AWT family. "Verdana Bold.ttf" (macOS),
     * "verdanab.ttf" (Windows), "Verdana-Regular.ttf" (Linux). A bold request falls back to the
     * regular cut.
     */
    fun findSystemFontFile(family: String, wantBold: Boolean): File? {
        val base = normalizeName(family)
        if (base.isEmpty()) return null
        val boldKeys = listOf("${base}bold", "${base}bd", "${base}b")
        val regularKeys = listOf(base, "${base}regular")
        val keys = if (wantBold) boldKeys + regularKeys else regularKeys
        for (key in keys) {
            systemFontFileIndex.find(key)?.let { return it }
        }
        return null
    }

    // ── Internals ─────────────────────────────────────────────────────────────

    private fun registerBundledFonts() {
        if (!bundledFontsRegistered.compareAndSet(false, true)) return
        for (name in BUNDLED_FONTS) {
            SlideFontRegistry::class.java.getResourceAsStream("/slide-fonts/$name")?.let { registerFontStream(it) }
        }
    }

    private fun refreshAvailableFamilies() {
        registerBundledFonts()
        try {
            val graphics = GraphicsEnvironment.getLocalGraphicsEnvironment()
            fontLocales.forEach { locale ->
                graphics.getAvailableFontFamilyNames(locale).forEach {
                    availableFamilies.putIfAbsent(it.lowercase(), it)
                }
            }
        } catch (_: Throwable) {
            // Headless environments without fontconfig can throw; the registry then only knows
            // explicitly registered fonts, which is still correct behavior.
            //
            // [Throwable] rather than [Exception] because the platform's native font manager
            // failing to load raises an `ExceptionInInitializerError`, which is an [Error] — and
            // this call is made from a startup thread, so not catching it ends the app.
        }
    }

    private fun registerFileIfUnknown(fontFile: File) {
        try {
            if (fontFile.extension.equals("dfont", ignoreCase = true)) {
                forEachMacFontResource(fontFile) { registerFontStream(it) }
                return
            }
            val fonts = Font.createFonts(fontFile)
            for (font in fonts) {
                if (!availableFamilies.containsKey(font.family.lowercase())) {
                    GraphicsEnvironment.getLocalGraphicsEnvironment().registerFont(font)
                }
                indexFamily(font)
            }
        } catch (_: Exception) {
            // Broken/unsupported font file — skip it.
        }
    }

    private fun indexFamily(font: Font) {
        for (locale in fontLocales) {
            availableFamilies.putIfAbsent(font.getFamily(locale).lowercase(), font.family)
        }
    }

    private fun cleanTypeface(name: String): String = name.substringBefore('\u0000').trim().removePrefix("@")

    private fun normalizeName(name: String): String =
        name.lowercase().filter { it.isLetterOrDigit() }

    /**
     * POI font handler that swaps unavailable families for the best installed substitute before
     * POI builds its AWT font. Everything else defers to POI's default behavior.
     */
    private class SubstitutingFontManager : DrawFontManagerDefault() {
        override fun getFallbackFont(graphics: Graphics2D, fontInfo: FontInfo?): FontInfo =
            FontInfo { Font.SANS_SERIF }
        /**
         * [fontInfo] is nullable on purpose: POI's own callers hand this handler a null for a run
         * that names no font of its own, and the Kotlin non-null check on the parameter then threw
         * before a single line of this method ran — taking the whole slide render down rather than
         * falling back. A run with no font asked for nothing in particular, so it gets
         * [DEFAULT_FAMILY] and goes through the same substitution path as everything else.
         */
        override fun getMappedFont(graphics: Graphics2D, fontInfo: FontInfo?): FontInfo {
            val mapped = super.getMappedFont(graphics, fontInfo ?: FontInfo { DEFAULT_FAMILY })
            val typeface = mapped.typeface ?: return mapped
            val substitute = availableFamilies[cleanTypeface(typeface).lowercase()] ?: resolveFamily(typeface)
            if (substitute == typeface) return mapped
            return object : FontInfo by mapped {
                override fun getTypeface(): String = substitute
            }
        }
    }
}
