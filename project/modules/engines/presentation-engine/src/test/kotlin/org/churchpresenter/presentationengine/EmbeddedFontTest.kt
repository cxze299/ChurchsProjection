package org.churchpresenter.presentationengine

import org.apache.poi.hslf.usermodel.HSLFSlideShow
import org.churchpresenter.presentationengine.fonts.SlideFontRegistry
import org.churchpresenter.presentationengine.fonts.forEachMacFontResource
import org.churchpresenter.presentationengine.pptx.PowerPointDeckSupport
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.file.Files
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Fonts a deck carries inside it: a usable one is registered, and one that cannot be read -- an
 * obfuscated or embed-restricted payload -- is skipped without failing the deck.
 */
class EmbeddedFontTest {

    private lateinit var dir: File

    @BeforeTest
    fun setUp() {
        dir = Files.createTempDirectory("cp-embedded-font").toFile()
    }

    @AfterTest
    fun tearDown() {
        dir.deleteRecursively()
    }

    @Test
    fun `a deck's embedded fonts are registered and a corrupt one is skipped`() {
        val pptx = Fixtures.createPptx(dir, listOf("Body" to "Notes"))
        val font = checkNotNull(javaClass.getResourceAsStream("/fonts/OpenSans-Regular.ttf")).use { it.readBytes() }
        Fixtures.addEmbeddedFontPart(pptx, "font1.fntdata", font)
        Fixtures.addEmbeddedFontPart(pptx, "font2.fntdata", byteArrayOf(1, 2, 3, 4))

        val slides = PowerPointDeckSupport.open(pptx).use { it.slides.size }

        assertEquals(1, slides, "the corrupt font must not fail the deck")
        assertTrue(SlideFontRegistry.isFamilyAvailable("Open Sans"))
    }

    @Test
    fun `a Mac data fork exposes the embedded sfnt face`() {
        val font = checkNotNull(javaClass.getResourceAsStream("/fonts/OpenSans-Regular.ttf")).use { it.readBytes() }
        val dataStart = 256
        val mapStart = dataStart + 4 + font.size
        val mapSize = 50
        val data = ByteBuffer.allocate(mapStart + mapSize)
        data.putInt(dataStart).putInt(mapStart).putInt(4 + font.size).putInt(mapSize)
        data.position(dataStart)
        data.putInt(font.size).put(font)
        data.position(mapStart + 24)
        data.putShort(28).putShort(50)
        data.putShort(0).putInt(0x73666E74).putShort(0).putShort(10)
        data.putShort(128).putShort(-1).putInt(0).putInt(0)
        val dfont = dir.resolve("OpenSans.dfont").apply { writeBytes(data.array()) }

        val families = mutableListOf<String?>()
        forEachMacFontResource(dfont) { families += SlideFontRegistry.registerFontStream(it) }
        assertEquals("Open Sans", families.single())
        assertEquals("Open Sans", SlideFontRegistry.resolveFamily("@Open Sans\u0000"))
    }

    @Test
    fun `binary PPS loads XOR encoded embedded fonts`() {
        val pps = dir.resolve("Hymn.pps")
        val font = checkNotNull(javaClass.getResourceAsStream("/fonts/OpenSans-Regular.ttf")).use { it.readBytes() }
        val names = listOf("Open Sans", "Regular", "Version 1.0", "Open Sans Regular")
            .map { it.toByteArray(Charsets.UTF_16LE) }
        val namesOffset = 80
        val headerSize = namesOffset + names.sumOf { 4 + it.size }
        val eot = ByteBuffer.allocate(headerSize + font.size).order(ByteOrder.LITTLE_ENDIAN)
        eot.putInt(eot.capacity())
        eot.putInt(font.size)
        eot.putInt(0x00010000)
        eot.putInt(0x10000000)
        eot.putInt(28, 400)
        eot.putShort(34, 0x504C)
        eot.position(namesOffset)
        names.forEach { name ->
            eot.putShort(0)
            eot.putShort(name.size.toShort())
            eot.put(name)
        }
        font.forEach { eot.put((it.toInt() xor 0x50).toByte()) }
        HSLFSlideShow().use { show ->
            show.addFont(eot.array().inputStream())
            show.createSlide().createTextBox().apply {
                text = "Hymn notation"
                textParagraphs.single().textRuns.single().fontFamily = "Open Sans"
            }
            pps.outputStream().use(show::write)
        }

        PowerPointDeckSupport.open(pps).use { show ->
            assertTrue(show is HSLFSlideShow)
            assertEquals(1, show.slides.size)
            assertEquals("Open Sans", SlideFontRegistry.resolveFamily("Open Sans"))
        }
    }
}
