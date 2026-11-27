package org.churchpresenter.presentationengine

import org.apache.poi.hslf.usermodel.HSLFSlideShow
import org.churchpresenter.presentationengine.fonts.SlideFontRegistry
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
