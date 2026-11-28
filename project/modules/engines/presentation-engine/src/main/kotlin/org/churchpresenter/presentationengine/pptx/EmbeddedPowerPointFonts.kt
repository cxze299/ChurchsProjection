package org.churchpresenter.presentationengine.pptx

import org.apache.poi.hslf.record.FontEmbeddedData
import org.churchpresenter.presentationengine.fonts.CompressedEmbeddedFont
import org.churchpresenter.presentationengine.fonts.SlideFontRegistry
import java.awt.Font
import java.awt.RenderingHints
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.InputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Rendering hint carrying the current deck's isolated embedded fonts (`normalized family -> faces`).
 * Bypassing `GraphicsEnvironment.registerFont`'s global no-op for duplicate family names ensures
 * two decks that each embed different subsets of the same musical score or CJK font never pollute
 * each other.
 */
internal val DECK_EMBEDDED_FONTS_KEY: RenderingHints.Key = object : RenderingHints.Key(0x4350_4546) {
    override fun isCompatibleValue(value: Any?): Boolean = value == null || value is Map<*, *>
}

internal object EmbeddedPowerPointFonts {
    private const val RECORD_HEADER_SIZE = 8
    private const val EOT_MAGIC_OFFSET = 34
    private const val EOT_MAGIC = 0x504C
    private const val EOT_MIN_HEADER_SIZE = 82
    private const val EOT_MAX_BYTES = 16 * 1024 * 1024
    private const val EOT_COMPRESSED = 0x00000004
    private const val EOT_XOR_ENCRYPTED = 0x10000000
    private const val EOT_XOR_KEY = 0x50
    private const val ODTTF_XOR_PREFIX_BYTES = 32
    private const val GUID_BYTE_LENGTH = 16
    private const val GUID_HEX_LENGTH = 32
    private val GUID_HEX_REGEX = Regex("[0-9a-fA-F]{8}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{12}")

    fun register(record: FontEmbeddedData) {
        loadFonts(record)
    }

    fun register(stream: InputStream, partName: String? = null) {
        loadFonts(stream, partName)
    }

    fun loadFonts(record: FontEmbeddedData): List<Font> {
        val serialized = ByteArrayOutputStream().also(record::writeOut).toByteArray()
        if (serialized.size <= RECORD_HEADER_SIZE) return emptyList()
        return loadFonts(ByteArrayInputStream(serialized, RECORD_HEADER_SIZE, serialized.size - RECORD_HEADER_SIZE))
    }

    fun loadFonts(stream: InputStream, partName: String? = null): List<Font> {
        val payload = stream.use { it.readBytes() }
        if (payload.isEmpty()) return emptyList()
        val fontStream = decodeFontBytes(payload, partName) ?: return emptyList()
        return createAndRegister(fontStream)
    }

    private fun decodeFontBytes(payload: ByteArray, partName: String?): InputStream? {
        val headerSize = EOT_MAGIC_OFFSET + Short.SIZE_BYTES
        if (payload.size >= headerSize) {
            val header = ByteBuffer.wrap(payload, 0, headerSize).order(ByteOrder.LITTLE_ENDIAN)
            if (header.getShort(EOT_MAGIC_OFFSET).toInt() == EOT_MAGIC) {
                return decodeEot(payload, header)
            }
        }
        if (partName != null && partName.endsWith(".odttf", ignoreCase = true)) {
            deobfuscateOdttf(partName, payload)?.let { return ByteArrayInputStream(it) }
        }
        return ByteArrayInputStream(payload)
    }

    private fun decodeEot(payload: ByteArray, header: ByteBuffer): InputStream? {
        val eotSize = header.int
        val fontSize = header.int
        header.int // EOT version
        val flags = header.int
        if (eotSize !in EOT_MIN_HEADER_SIZE..EOT_MAX_BYTES ||
            fontSize <= 0 || fontSize > eotSize - EOT_MIN_HEADER_SIZE ||
            payload.size < eotSize
        ) return null
        val eot = if (payload.size == eotSize) payload.copyOf() else payload.copyOfRange(0, eotSize)
        if (flags and EOT_COMPRESSED != 0) {
            return CompressedEmbeddedFont.decode(eot)
        }
        val start = eotSize - fontSize
        if (flags and EOT_XOR_ENCRYPTED != 0) {
            for (index in start until eotSize) {
                eot[index] = (eot[index].toInt() xor EOT_XOR_KEY).toByte()
            }
        }
        return ByteArrayInputStream(eot, start, fontSize)
    }

    /**
     * ECMA-376 Part 2 §14.2.9 OOXML font obfuscation (`.odttf`):
     * XOR the first 32 bytes twice with the 16-byte GUID extracted from the part name in reverse byte order.
     */
    internal fun deobfuscateOdttf(partName: String, payload: ByteArray): ByteArray? {
        if (payload.size < ODTTF_XOR_PREFIX_BYTES) return null
        val match = GUID_HEX_REGEX.find(partName) ?: return null
        val hex = match.value.replace("-", "")
        if (hex.length != GUID_HEX_LENGTH) return null
        val key = ByteArray(GUID_BYTE_LENGTH)
        for (i in 0 until GUID_BYTE_LENGTH) {
            val hexIndex = (GUID_BYTE_LENGTH - 1 - i) * 2
            key[i] = hex.substring(hexIndex, hexIndex + 2).toInt(16).toByte()
        }
        val decoded = payload.copyOf()
        for (i in 0 until ODTTF_XOR_PREFIX_BYTES) {
            decoded[i] = (decoded[i].toInt() xor key[i % GUID_BYTE_LENGTH].toInt()).toByte()
        }
        return decoded
    }

    private fun createAndRegister(stream: InputStream): List<Font> = try {
        val fonts = stream.use { Font.createFonts(it) }.toList()
        fonts.forEach { SlideFontRegistry.registerFont(it) }
        fonts
    } catch (_: Exception) {
        emptyList()
    }
}
