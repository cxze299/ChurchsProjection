package org.churchpresenter.presentationengine

import org.apache.poi.hslf.usermodel.HSLFSlideShow
import org.apache.poi.sl.draw.DrawFactory
import org.apache.poi.sl.draw.DrawTextShape
import org.apache.poi.sl.draw.Drawable
import org.apache.poi.util.Units
import org.apache.poi.xslf.usermodel.XMLSlideShow
import org.apache.xmlbeans.XmlObject
import org.churchpresenter.presentationengine.fonts.SlideFontRegistry
import org.churchpresenter.presentationengine.pptx.EmbeddedPowerPointFonts
import org.churchpresenter.presentationengine.pptx.PowerPointDeckSupport
import org.churchpresenter.presentationengine.pptx.SlideDrawFactory
import org.churchpresenter.presentationengine.pptx.preferEquationFallbacks
import org.openxmlformats.schemas.presentationml.x2006.main.CTShape
import java.awt.Color
import java.awt.Font
import java.awt.Rectangle
import java.awt.image.BufferedImage
import java.io.File
import java.nio.charset.Charset
import java.nio.file.Files
import java.util.concurrent.atomic.AtomicInteger
import javax.xml.namespace.QName
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class PowerPointScoreAndDualTrackTest {

    private lateinit var temp: File
    private lateinit var originalCacheBase: File

    @BeforeTest
    fun setUp() {
        temp = Files.createTempDirectory("cp-dual-track-score").toFile()
        originalCacheBase = PowerPointDocumentPdf.cacheBaseDir
        PowerPointDocumentPdf.cacheBaseDir = File(temp, "pdf-cache")
        PowerPointDocumentPdf.clearFailureCache()
        PowerPointDocumentPdf.converterOverride = null
    }

    @AfterTest
    fun tearDown() {
        PowerPointDocumentPdf.converterOverride = null
        PowerPointDocumentPdf.cacheBaseDir = originalCacheBase
        PowerPointDocumentPdf.clearFailureCache()
        temp.deleteRecursively()
    }

    @Test
    fun `Track A uses cached native PDF for static PPS decks and falls back on mismatch`() {
        val pps = File(temp, "hymn.pps")
        HSLFSlideShow().use { ppt ->
            ppt.createSlide().createAutoShape().apply {
                anchor = Rectangle(20, 20, 120, 80)
                fillColor = Color.BLUE
            }
            pps.outputStream().use(ppt::write)
        }

        val converterCalls = AtomicInteger(0)
        PowerPointDocumentPdf.converterOverride = { _, workDir ->
            converterCalls.incrementAndGet()
            Fixtures.createPdf(workDir, pages = 1, name = "exported.pdf")
        }

        val deck = assertIs<LoadResult.Success>(PresentationLoader.load(pps)).deck
        DeckRasterizer(deck, targetWidthPx = 320).use { rasterizer ->
            assertEquals(320, rasterizer.renderFinalFrame(0).width)
        }
        // Second open must hit the disk PDF cache without calling the converter again.
        DeckRasterizer(deck, targetWidthPx = 320).use { rasterizer ->
            assertEquals(320, rasterizer.renderFinalFrame(0).width)
        }
        assertEquals(1, converterCalls.get(), "cached PDF should avoid repeated Office conversions")

        // A mismatched PDF page count must fall back seamlessly to the in-JVM POI renderer.
        PowerPointDocumentPdf.cacheBaseDir = File(temp, "pdf-cache-mismatch")
        PowerPointDocumentPdf.converterOverride = { _, workDir ->
            Fixtures.createPdf(workDir, pages = 3, name = "wrong-count.pdf")
        }
        DeckRasterizer(deck, targetWidthPx = 320).use { rasterizer ->
            assertEquals(320, rasterizer.renderFinalFrame(0).width)
        }
    }

    @Test
    fun `OOXML odttf GUID obfuscated embedded fonts are decoded and indexed per deck`() {
        val fontBytes = checkNotNull(javaClass.getResourceAsStream("/fonts/OpenSans-Regular.ttf")).use { it.readBytes() }
        val guid = "12345678-9ABC-DEF0-1234-56789ABCDEF0"
        val obfuscated = checkNotNull(EmbeddedPowerPointFonts.deobfuscateOdttf("$guid.odttf", fontBytes))
        // XOR obfuscation is symmetric: applying deobfuscation twice restores the original bytes.
        val restored = checkNotNull(EmbeddedPowerPointFonts.deobfuscateOdttf("/ppt/fonts/{$guid}.odttf", obfuscated))
        assertTrue(fontBytes.contentEquals(restored))

        val pptx = Fixtures.createPptx(temp, listOf("1 3 5 5" to ""))
        Fixtures.addEmbeddedFontPart(pptx, "$guid.odttf", obfuscated)
        PowerPointDeckSupport.open(pptx).use { show ->
            val deckFonts = PowerPointDeckSupport.embeddedFontsFor(show)
            assertTrue(deckFonts.containsKey("open sans"), "per-deck map must contain decoded odttf font")
        }
    }

    @Test
    fun `explicit horizontal tab stops advance text past the configured tab coordinate`() {
        fun rightmostInkX(withExplicitTab: Boolean): Int {
            XMLSlideShow().use { ppt ->
                val box = ppt.createSlide().createTextBox()
                box.anchor = Rectangle(0, 0, 500, 80)
                box.leftInset = 0.0
                box.rightInset = 0.0
                box.clearText()
                val paragraph = box.addNewTextParagraph()
                if (withExplicitTab) {
                    val pPr = paragraph.xmlObject.pPr ?: paragraph.xmlObject.addNewPPr()
                    val tab = pPr.addNewTabLst().addNewTab()
                    tab.pos = Units.toEMU(260.0)
                }
                val run = paragraph.addNewTextRun()
                run.setText(if (withExplicitTab) "1\t5" else "1 5")
                run.fontSize = 20.0
                run.fontFamily = Font.SANS_SERIF

                val canvas = BufferedImage(500, 80, BufferedImage.TYPE_INT_ARGB)
                val graphics = canvas.createGraphics()
                try {
                    graphics.setRenderingHint(Drawable.DRAW_FACTORY, SlideDrawFactory)
                    DrawFactory.getInstance(graphics).getDrawable(box).draw(graphics)
                } finally {
                    graphics.dispose()
                }
                return (canvas.width - 1 downTo 0).firstOrNull { x ->
                    (0 until canvas.height).any { y -> canvas.getRGB(x, y) ushr 24 != 0 }
                } ?: -1
            }
        }

        val plainX = rightmostInkX(withExplicitTab = false)
        val tabbedX = rightmostInkX(withExplicitTab = true)
        assertTrue(tabbedX >= 255, "tabbed glyph should align near 260pt tab stop (got $tabbedX)")
        assertTrue(tabbedX > plainX + 180, "tabbed text ($tabbedX) should advance well past plain ($plainX)")
    }

    @Test
    fun `normAutofit fontScale and lnSpcReduction shrink lyric text height`() {
        fun measureHeight(applyAutofit: Boolean): Double {
            XMLSlideShow().use { ppt ->
                val box = ppt.createSlide().createTextBox()
                box.anchor = Rectangle(0, 0, 400, 200)
                box.clearText()
                repeat(3) {
                    box.addNewTextParagraph().addNewTextRun().apply {
                        setText("奇异恩典何等甘甜")
                        fontSize = 24.0
                        fontFamily = Font.SANS_SERIF
                    }
                }
                if (applyAutofit) {
                    val bodyPr = (box.xmlObject as CTShape).txBody.bodyPr
                    val autofit = bodyPr.addNewNormAutofit()
                    autofit.fontScale = 60_000
                    autofit.lnSpcReduction = 20_000
                }
                val graphics = BufferedImage(1, 1, BufferedImage.TYPE_INT_ARGB).createGraphics()
                try {
                    graphics.setRenderingHint(Drawable.DRAW_FACTORY, SlideDrawFactory)
                    val drawable = DrawFactory.getInstance(graphics).getDrawable(box) as DrawTextShape
                    return drawable.getTextHeight(graphics)
                } finally {
                    graphics.dispose()
                }
            }
        }

        val normal = measureHeight(applyAutofit = false)
        val shrunk = measureHeight(applyAutofit = true)
        assertTrue(shrunk < normal * 0.65, "autofit shrunk height ($shrunk) must be well below normal ($normal)")
    }

    @Test
    fun `legacy binary PPT notation accent box lifts baseline inside tight overlay anchor`() {
        HSLFSlideShow().use { ppt ->
            val slide = ppt.createSlide()
            val box = slide.createTextBox()
            box.anchor = Rectangle(10, 40, 120, 14)
            box.topInset = 0.0
            box.bottomInset = 0.0
            box.text = "A"
            val run = box.textParagraphs.single().textRuns.single()
            run.fontSize = 36.0
            run.fontFamily = "SimpMusic Accent"

            val canvas = BufferedImage(160, 100, BufferedImage.TYPE_INT_ARGB)
            val graphics = canvas.createGraphics()
            try {
                graphics.setRenderingHint(Drawable.DRAW_FACTORY, SlideDrawFactory)
                DrawFactory.getInstance(graphics).getDrawable(box).draw(graphics)
            } finally {
                graphics.dispose()
            }
            val topInk = (0 until canvas.height).firstOrNull { y ->
                (0 until canvas.width).any { x -> canvas.getRGB(x, y) ushr 24 != 0 }
            }
            assertNotNull(topInk, "notation glyph should render")
            assertTrue(topInk < 40, "lifted baseline should draw above the tight box bottom (topInk=$topInk)")
        }
    }

    @Test
    fun `WMF GB18030 mojibake Chinese font names are recovered cleanly`() {
        val gb18030 = Charset.forName("GB18030")
        val mojibakeSong = String("宋体".toByteArray(gb18030), Charsets.ISO_8859_1)
        val mojibakeYahei = String("微软雅黑".toByteArray(gb18030), Charsets.ISO_8859_1)
        assertEquals("宋体", SlideFontRegistry.cleanTypeface(mojibakeSong))
        assertEquals("微软雅黑", SlideFontRegistry.cleanTypeface(mojibakeYahei))
    }

    @Test
    fun `equation fallback also promotes grouped shapes for fractional time signatures`() {
        val xml = XmlObject.Factory.parse(
            """
            <p:spTree xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                      xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"
                      xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
                <mc:AlternateContent>
                    <mc:Choice Requires="m"><p:sp><m:oMath/></p:sp></mc:Choice>
                    <mc:Fallback><p:grpSp id="time-sig"/></mc:Fallback>
                </mc:AlternateContent>
            </p:spTree>
            """.trimIndent()
        )
        preferEquationFallbacks(xml)
        xml.newCursor().use { cursor ->
            cursor.toFirstChild()
            cursor.toFirstChild()
            assertEquals("grpSp", cursor.name.localPart)
            assertEquals("time-sig", cursor.getAttributeText(QName("id")))
        }
    }
}
