package org.churchpresenter.bibletab

import org.churchpresenter.core.models.bible.SelectedVerse

/** One accumulated text block per translation, so existing projection layouts still apply. */
internal fun BibleTabScope.publishPassage(verses: List<SelectedVerse>) {
    if (!ui.appendPassages || verses.isEmpty()) {
        onVerseSelected(verses)
        return
    }
    if (ui.appendedPassages.lastOrNull() != verses) {
        ui.appendedPassages = ui.appendedPassages + listOf(verses)
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
