package org.churchpresenter.slides.viewmodel

import org.churchpresenter.presentationengine.PresentationLoader
import org.churchpresenter.core.models.songs.HymnInitials
import org.churchpresenter.core.models.songs.songSearchText
import java.io.File
import java.text.Normalizer

internal object HymnQuickPlay {
    private const val OTHER_FORMAT_RANK = 4
    private val numberSegment = Regex("[0-9]+")
    private val leadingNumber = Regex("^\\s*(?:第|[\\[【（(])?\\s*([0-9]+)")
    private val words = Regex("[\\s\\p{Z}]+")
    private val formatPrefix = Regex("^(?:\\*?\\.)?([a-z0-9]+)[\\s\\p{Z}]+", RegexOption.IGNORE_CASE)
    private val formatSuffix = Regex("(?:^|[\\s\\p{Z}]+|\\.)(?:\\*?\\.)?([a-z0-9]+)$", RegexOption.IGNORE_CASE)
    private val powerPointFormats =
        PresentationLoader.POWERPOINT_XML_EXTENSIONS + PresentationLoader.POWERPOINT_BINARY_EXTENSIONS

    fun book(file: File, root: File, roots: List<File> = listOf(root)): String {
        val parent = file.absoluteFile.toPath().normalize().parent ?: return ""
        val owner = roots.asSequence().map { it.absoluteFile.toPath().normalize() }
            .filter { parent.startsWith(it) }.maxByOrNull { it.nameCount }
            ?: root.absoluteFile.toPath().normalize().takeIf { parent.startsWith(it) }
            ?: return ""
        val name = if (owner == parent) {
            owner.fileName?.toString().orEmpty()
        } else {
            parent.getName(owner.nameCount).toString()
        }
        return normalize(name)
    }

    fun find(files: List<File>, root: File, query: String, preferredBook: String, format: String,
        roots: List<File> = listOf(root),
    ): List<File> {
        val normalizedQuery = normalize(query.trim())
        val suffix = formatSuffix.find(normalizedQuery)?.takeIf {
            it.groupValues[1].lowercase() in PresentationLoader.SUPPORTED_EXTENSIONS
        }
        val prefix = formatPrefix.find(normalizedQuery)?.takeIf {
            it.groupValues[1].lowercase() in PresentationLoader.SUPPORTED_EXTENSIONS
        }
        val extension = (suffix ?: prefix)?.groupValues?.get(1)?.lowercase()
        val text = when {
            suffix != null -> normalizedQuery.substring(0, suffix.range.first).trimEnd()
            prefix != null -> normalizedQuery.substring(prefix.range.last + 1)
            else -> normalizedQuery
        }
        val normalizedText = songSearchText(text)
        val tokens = normalizedText.split(words)
        val numbers = tokens.filter { numberSegment.matches(it) }.map { it.trimStart('0') }
        val titleWords = tokens.filterNot { numberSegment.matches(it) }
        val bookPreference = normalize(preferredBook)
        val matches = files.filter { file ->
            if (extension != null && !file.extension.equals(extension, ignoreCase = true)) return@filter false
            val title = normalize(file.nameWithoutExtension)
            if (numbers.isNotEmpty() && !numbers.all { number ->
                    numberSegment.findAll(title).any { it.value.trimStart('0') == number }
                }) return@filter false
            if (titleWords.isEmpty() || normalizedText.isEmpty()) return@filter true
            val searchableTitle = songSearchText(title)
            searchableTitle.contains(normalizedText, ignoreCase = true) ||
                HymnInitials.matches(title, titleWords.joinToString("")) ||
                titleWords.all { searchableTitle.contains(it, ignoreCase = true) }
        }
        val preferredMatches = if (normalizedQuery.isEmpty() && bookPreference.isNotEmpty()) {
            matches.filter { book(it, root, roots) == bookPreference }.ifEmpty { matches }
        } else {
            matches
        }
        val grouped = preferredMatches.groupBy {
            it.absoluteFile.toPath().normalize().parent to normalize(it.nameWithoutExtension).lowercase()
        }
        val preferredFormat = format.lowercase()
        return grouped.values.map { variants ->
            variants.minWith(
                compareBy<File> { formatRank(it.extension.lowercase(), preferredFormat) }.thenBy { it.absolutePath }
            )
        }.sortedWith(
            compareBy<File> { !normalize(it.nameWithoutExtension).equals(text, ignoreCase = true) }
                .thenBy { bookPreference.isNotEmpty() && book(it, root, roots) != bookPreference }
                .thenBy {
                    numbers.isNotEmpty() &&
                        leadingNumber.find(it.nameWithoutExtension)?.groupValues?.get(1)?.trimStart('0') !in numbers
                }
                .thenBy { normalize(it.name) }
                .thenBy { it.absolutePath }
        )
    }

    private fun normalize(text: String): String = Normalizer.normalize(text, Normalizer.Form.NFC)

    private fun formatRank(extension: String, preferred: String): Int = when {
        extension == preferred -> 0
        preferred in powerPointFormats && extension in powerPointFormats -> 1
        preferred in PresentationLoader.WORD_EXTENSIONS && extension in PresentationLoader.WORD_EXTENSIONS -> 1
        extension == "pptx" -> 2
        extension == "pdf" -> 3
        else -> OTHER_FORMAT_RANK
    }
}
