package org.churchpresenter.bibletab

import org.churchpresenter.core.models.bible.SelectedVerse

/** One accumulated text block per translation, so existing projection layouts still apply. */
internal fun BibleTabScope.publishPassage(verses: List<SelectedVerse>) {
    if (!ui.appendPassages || verses.isEmpty()) {
        onVerseSelected(verses)
        return
    }
    if (ui.appendedPassages.lastOrNull() != verses) {
        val retained = ui.appendedPassages.mapNotNull { previous ->
            previous.filterNot { single ->
                val (start, end) = verseSpan(single.verseRange, single.verseNumber)
                end == null && verses.any { passage ->
                    val (rangeStart, rangeEnd) = verseSpan(passage.verseRange, passage.verseNumber)
                    rangeEnd != null && rangeEnd > rangeStart &&
                        single.translationFileName == passage.translationFileName &&
                        single.bibleName == passage.bibleName && single.bookName == passage.bookName &&
                        single.chapter == passage.chapter && start in rangeStart..rangeEnd
                }
            }.takeIf { it.isNotEmpty() }
        }
        ui.appendedPassages = retained + listOf(verses)
    }
    val combined = ui.appendedPassages.flatten()
        .groupBy { it.translationFileName.ifBlank { it.bibleName } }
        .values.map { passages ->
            passages.last().copy(verseText = passages.joinToString("\n\n") { verse ->
                val reference = verse.verseRange.ifBlank { verse.verseNumber.toString() }
                "${verse.bookName} ${verse.chapter}:$reference\n${verse.verseText}"
            })
        }
    onVerseSelected(combined)
}
