package org.churchpresenter.bibletab

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import org.churchpresenter.core.models.bible.SelectedVerse
import org.churchpresenter.sharedui.models.Presenting

private val PASSAGE_BOOK = Regex("[a-zA-Z\\u4e00-\\u9fff]+\\s+(?=\\d)")
private val PASSAGE_NUMBERS = Regex("(\\d+)\\s+(\\d+)(?:\\s*[- ]\\s*(\\d+))?\\s*")

internal fun isQuickPassageInput(query: String): Boolean = PASSAGE_BOOK.find(query.trim())?.range?.first == 0

/** Parse the complete input, including a book immediately following the preceding verse number. */
internal fun BibleViewModel.quickPassageReferences(query: String): List<SmartReference>? {
    val text = query.trim()
    val books = PASSAGE_BOOK.findAll(text).toList()
    if (books.isEmpty() || books.first().range.first != 0) return null
    return books.mapIndexed { index, book ->
        val end = books.getOrNull(index + 1)?.range?.first ?: text.length
        val numbers = PASSAGE_NUMBERS.matchEntire(text.substring(book.range.last + 1, end)) ?: return null
        val chapter = numbers.groupValues[1].toIntOrNull()?.takeIf { it > 0 } ?: return null
        val first = numbers.groupValues[2].toIntOrNull()?.takeIf { it > 0 } ?: return null
        val lastToken = numbers.groupValues[3]
        val last = if (lastToken.isEmpty()) first else lastToken.toIntOrNull()?.takeIf { it > 0 } ?: return null
        parseReference("${book.value.trim()} $chapter:${minOf(first, last)}-${maxOf(first, last)}") ?: return null
    }
}

internal fun BibleViewModel.quickPassageLabel(query: String): String? = quickPassageReferences(query)?.joinToString("；") { ref ->
    val start = ref.verseStart
    val end = ref.verseEnd
    "${_books.value.getOrNull(ref.bookIndex).orEmpty()} ${ref.chapter}章" +
        if (start == end) "${start}节" else "$start-${end}节"
}

internal fun BibleViewModel.updateQuickPassageQuery(query: String) {
    searchJob?.cancel()
    loadChapterJob?.cancel()
    _searchQuery.value = query
    _isSearchMode.value = false
    _searchResults.value = emptyList()
}

/** Validate the whole request before changing the selection or publishing any of its passages. */
internal suspend fun BibleViewModel.resolveQuickPassages(refs: List<SmartReference>): List<List<SelectedVerse>>? {
    awaitFullyLoaded()
    val bible = _primaryBible.value ?: return null
    val valid = withContext(ioDispatcher) {
        refs.all { ref ->
            ref.bookIndex in _books.value.indices && containsRequestedVerses(
                bible.getChapter(bible.getBookId(ref.bookIndex), ref.chapter ?: 1).verses, ref,
            )
        }
    }
    if (!valid) return null
    loadChapterJob?.cancel()
    searchJob?.cancel()
    return refs.map { ref ->
        resolveVerseSelection(_books.value[ref.bookIndex], ref.chapter ?: 1, ref.verseStart ?: 1,
            "${ref.verseStart}-${ref.verseEnd ?: ref.verseStart}")
        selectedVersesUnsplit().takeIf { it.isNotEmpty() } ?: return null
    }
}

internal fun BibleTabScope.submitSearch(viewModel: BibleViewModel) {
    if (ui.appendPassages && searchMode != BibleSearchMode.TEXT && isQuickPassageInput(searchQuery)) {
        ui.quickInputSuppressedQuery = null
        ui.quickInputRequest++
    } else viewModel.submitSmartQuery()
}

@Composable
internal fun BibleTabScope.BibleQuickPassageEffect(viewModel: BibleViewModel) {
    LaunchedEffect(searchQuery, ui.appendPassages, searchMode, ui.quickInputRequest, translationSelectionKey) {
        if (!ui.appendPassages || searchMode == BibleSearchMode.TEXT) return@LaunchedEffect
        val query = searchQuery
        if (ui.quickInputSuppressedQuery == query) return@LaunchedEffect
        if (searchQuery.isBlank()) {
            ui.quickInputBase = null
            return@LaunchedEffect
        }
        val refs = viewModel.quickPassageReferences(query) ?: return@LaunchedEffect
        delay(300)
        val passages = viewModel.resolveQuickPassages(refs) ?: return@LaunchedEffect
        if (searchQuery != query || ui.quickInputSuppressedQuery == query) return@LaunchedEffect
        val base = ui.quickInputBase ?: ui.appendedPassages.also { ui.quickInputBase = it }
        ui.appendedPassages = passages.fold(base, ::appendPassage)
        publishAccumulatedPassages()
        bibleOutput?.setBibleHold(false)
        onInstanceLinkSendBibleHold?.invoke(false)
        onPresenting(Presenting.BIBLE)
    }
}
