package org.churchpresenter.presentationengine.pptx

import org.apache.poi.hslf.record.FontEmbeddedData
import org.churchpresenter.presentationengine.fonts.CompressedEmbeddedFont
import org.churchpresenter.presentationengine.fonts.SlideFontRegistry
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.InputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder

internal object EmbeddedPowerPointFonts {
    private const val RECORD_HEADER_SIZE = 8
    private const val EOT_MAGIC_OFFSET = 34
    private const val EOT_MAGIC = 0x504C
    private const val EOT_MIN_HEADER_SIZE = 82
    private const val EOT_MAX_BYTES = 16 * 1024 * 1024
    private const val EOT_COMPRESSED = 0x00000004
    private const val EOT_XOR_ENCRYPTED = 0x10000000
    private const val EOT_XOR_KEY = 0x50

    fun register(record: FontEmbeddedData) {
        val serialized = ByteArrayOutputStream().also(record::writeOut).toByteArray()
        register(ByteArrayInputStream(serialized, RECORD_HEADER_SIZE, serialized.size - RECORD_HEADER_SIZE))
    }

    fun register(stream: InputStream) {
        stream.buffered().use { input ->
            val headerSize = EOT_MAGIC_OFFSET + Short.SIZE_BYTES
            input.mark(headerSize)
            val bytes = input.readNBytes(headerSize)
            input.reset()
            val header = ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)
            if (bytes.size < headerSize || header.getShort(EOT_MAGIC_OFFSET).toInt() != EOT_MAGIC) {
                SlideFontRegistry.registerFontStream(input)
                return
            }
            val eotSize = header.int
            val fontSize = header.int
            header.int // EOT version
            val flags = header.int
            if (eotSize !in EOT_MIN_HEADER_SIZE..EOT_MAX_BYTES ||
                fontSize <= 0 || fontSize > eotSize - EOT_MIN_HEADER_SIZE
            ) return
            val eot = input.readNBytes(eotSize)
            if (eot.size != eotSize) return
            if (flags and EOT_COMPRESSED != 0) {
                CompressedEmbeddedFont.decode(eot)?.let(SlideFontRegistry::registerFontStream)
                return
            }
            // The OpenType payload is the final field in an EOT record.
            val start = eotSize - fontSize
            if (flags and EOT_XOR_ENCRYPTED != 0) {
                for (index in start until eotSize) {
                    eot[index] = (eot[index].toInt() xor EOT_XOR_KEY).toByte()
                }
            }
            SlideFontRegistry.registerFontStream(ByteArrayInputStream(eot, start, fontSize))
        }
    }
}
