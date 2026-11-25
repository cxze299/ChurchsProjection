package org.churchpresenter.presenter

import androidx.compose.foundation.LocalScrollbarStyle
import androidx.compose.foundation.VerticalScrollbar
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.rememberScrollbarAdapter
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import org.churchpresenter.core.models.bible.SelectedVerse
import org.churchpresenter.settings.BibleTranslationSettings
import org.churchpresenter.sharedui.utils.Utils.parseHexColor

/** A top-to-bottom reading surface using each translation's configured text and reference styles. */
@Composable
internal fun BibleScrollingPassages(slide: BibleSlide, verses: List<SelectedVerse>, modifier: Modifier) {
    with(slide) {
        val scroll = rememberScrollState()
        val allowed = if (isParallelIntended) maxOf(translationStack.size, 2) else 1
        val visible = verses.take(allowed)
        val passages = verses.firstOrNull { it.continuousPassages.isNotEmpty() }?.continuousPassages.orEmpty()
        Box(modifier.fillMaxSize().clipToBounds()) {
            Column(
                modifier = Modifier.fillMaxSize().testTag("continuous_bible_content")
                    .verticalScroll(scroll).padding(end = 12.dp),
                verticalArrangement = Arrangement.spacedBy(bs.multiTranslationSpacing.dp),
            ) {
                passages.forEach { translations ->
                    translations.forEach { verse ->
                        val index = visible.indexOfFirst {
                            it.translationFileName.ifBlank { it.bibleName } == verse.translationFileName.ifBlank { verse.bibleName }
                        }
                        if (index >= 0) {
                            val item = translationStack.firstOrNull { it.fileName == verse.translationFileName }
                                ?: translationStack.getOrNull(index)
                                ?: BibleTranslationSettings(fileName = verse.translationFileName)
                            TranslationBlock(verse, item, fitScale = 1f, honorTextBoxes = false)
                        }
                    }
                }
            }
            if (scroll.maxValue > 0) {
                val color = if (isKey) Color.White else parseHexColor(
                    translationStack.firstOrNull()?.textColor ?: "#FFFFFF",
                )
                VerticalScrollbar(
                    adapter = rememberScrollbarAdapter(scroll),
                    modifier = Modifier.align(Alignment.CenterEnd).fillMaxHeight().testTag("continuous_bible_scrollbar"),
                    style = LocalScrollbarStyle.current.copy(
                        unhoverColor = color.copy(alpha = 0.45f), hoverColor = color.copy(alpha = 0.8f),
                    ),
                )
            }
        }
    }
}
