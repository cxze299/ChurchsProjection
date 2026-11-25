package org.churchpresenter.bibletab

import org.churchpresenter.core.models.bible.SelectedVerse

/** Clear the accumulated text and start the next mouse click as a new passage. */
internal fun BibleTabScope.clearPassages(viewModel: BibleViewModel) {
    ui.appendedPassages = emptyList()
    ui.quickInputBase = null
    ui.quickInputSuppressedQuery = searchQuery
    viewModel.clearMultiVerseSelection()
    viewModel.mouseRangeSelectionToken = null
    onClearVerses()
}

/** One accumulated text block per translation, so existing projection layouts still apply. */
internal fun BibleTabScope.publishPassage(verses: List<SelectedVerse>) {
    ui.quickInputBase = null
    ui.quickInputSuppressedQuery = searchQuery
    if (!ui.appendPassages || verses.isEmpty()) {
        onVerseSelected(verses)
        return
    }
    ui.appendedPassages = appendPassage(ui.appendedPassages, verses)
    publishAccumulatedPassages()
}

internal fun appendPassage(existing: List<List<SelectedVerse>>, verses: List<SelectedVerse>): List<List<SelectedVerse>> {
    if (existing.lastOrNull() == verses) return existing
    val retained = existing.mapNotNull { previous ->
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
    return retained + listOf(verses)
}

internal fun BibleTabScope.publishAccumulatedPassages() {
    val combined = ui.appendedPassages.flatten()
        .groupBy { it.translationFileName.ifBlank { it.bibleName } }
        .values.map { passages ->
            passages.last().copy(continuousPassages = ui.appendedPassages, verseText = passages.joinToString("\n\n") { verse ->
                val reference = verse.verseRange.ifBlank { verse.verseNumber.toString() }
                "${verse.bookName} ${verse.chapter}:$reference\n${verse.verseText}"
            })
        }
    onVerseSelected(combined)
}
