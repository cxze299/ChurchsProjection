package org.churchpresenter.presentationengine.fonts

import java.io.File
import java.io.InputStream
import java.io.RandomAccessFile

private const val SFNT_RESOURCE_TYPE = 0x73666E74
private const val TYPE_LIST_OFFSET = 24
private const val TYPE_ENTRY_SIZE = 8
private const val RESOURCE_ENTRY_SIZE = 12
private const val RESOURCE_DATA_MASK = 0xFFFFFF
private const val MAX_FONT_RESOURCE_SIZE = 32 * 1024 * 1024

/** Reads TrueType/OpenType resources from a macOS data-fork font container. */
internal fun forEachMacFontResource(file: File, onFont: (InputStream) -> Unit) {
    RandomAccessFile(file, "r").use { reader ->
        val dataStart = reader.readInt().toLong()
        val mapStart = reader.readInt().toLong()
        val dataEnd = dataStart + reader.readInt()
        val mapEnd = mapStart + reader.readInt()
        if (dataStart !in 16..dataEnd || mapStart !in 16..mapEnd ||
            dataEnd > reader.length() || mapEnd > reader.length() || mapStart + TYPE_LIST_OFFSET + 2 > mapEnd
        ) return
        reader.seek(mapStart + TYPE_LIST_OFFSET)
        val typesStart = mapStart + reader.readUnsignedShort()
        if (typesStart + 2 > mapEnd) return
        reader.seek(typesStart)
        val types = reader.readShort().toInt() + 1
        if (typesStart + 2 + types * TYPE_ENTRY_SIZE > mapEnd) return
        repeat(types.coerceAtLeast(0)) { typeIndex ->
            reader.seek(typesStart + 2 + typeIndex * TYPE_ENTRY_SIZE)
            val type = reader.readInt()
            val resourceCount = reader.readUnsignedShort() + 1
            val resourcesStart = typesStart + reader.readUnsignedShort()
            if (type != SFNT_RESOURCE_TYPE || resourcesStart + resourceCount * RESOURCE_ENTRY_SIZE > mapEnd) return@repeat
            for (index in 0 until resourceCount) {
                reader.seek(resourcesStart + index * RESOURCE_ENTRY_SIZE + 4)
                val payload = dataStart + (reader.readInt() and RESOURCE_DATA_MASK)
                if (payload + 4 > dataEnd) continue
                reader.seek(payload)
                val size = reader.readInt()
                if (size !in 1..MAX_FONT_RESOURCE_SIZE || payload + 4 + size > dataEnd) continue
                val bytes = ByteArray(size)
                reader.readFully(bytes)
                onFont(bytes.inputStream())
            }
        }
    }
}
