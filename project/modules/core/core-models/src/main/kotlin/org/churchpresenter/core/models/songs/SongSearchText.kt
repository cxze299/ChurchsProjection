package org.churchpresenter.core.models.songs

import com.github.houbb.opencc4j.util.ZhConverterUtil
import java.text.Normalizer

private val separatorsBetweenHan = Regex("(?<=\\p{IsHan})[\\p{Z}\\s\\p{P}]+(?=\\p{IsHan})")
private val hasHan = Regex("\\p{IsHan}")

fun containsHan(text: String): Boolean = hasHan.containsMatchIn(text)

/** Canonical comparison text for titles, leaving the displayed names untouched. */
fun songSearchText(text: String): String {
    val normalized = Normalizer.normalize(text, Normalizer.Form.NFC)
    if (!containsHan(normalized)) return normalized
    return ZhConverterUtil.toSimple(normalized.replace(separatorsBetweenHan, ""))
}
