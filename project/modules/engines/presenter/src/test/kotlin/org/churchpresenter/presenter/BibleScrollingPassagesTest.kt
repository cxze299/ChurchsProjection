@file:OptIn(androidx.compose.ui.test.ExperimentalTestApi::class)

package org.churchpresenter.presenter

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toAwtImage
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.v2.runComposeUiTest
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.unit.dp
import kotlinx.serialization.json.Json
import org.churchpresenter.core.models.bible.SelectedVerse
import org.churchpresenter.settings.AppSettings
import org.churchpresenter.settings.BibleSettings
import org.churchpresenter.settings.BibleTranslationSettings
import java.io.File
import javax.imageio.ImageIO
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class BibleScrollingPassagesTest {
    private fun passage(index: Int) = SelectedVerse(
        translationFileName = "test.spb", bookName = if (index % 2 == 0) "马太福音" else "启示录",
        chapter = index + 1, verseNumber = 1, verseRange = "1-3",
        verseText = "第${index + 1}段经文：上帝说，要有光，就有了光。",
    )

    private fun combined(count: Int) = passage(0).copy(continuousPassages = List(count) { listOf(passage(it)) })
    private val settings = AppSettings(bibleSettings = BibleSettings().withTranslations(listOf(
        BibleTranslationSettings(fileName = "test.spb", textFontSize = 52, referenceFontSize = 32),
    )))

    @Test
    fun `long passages retain font size and scroll from the first passage to the last`() = runComposeUiTest {
        val displayed = mutableStateOf(listOf(combined(1)))
        setContent {
            Box(Modifier.size(800.dp, 450.dp)) {
                BiblePresenter(selectedVerses = displayed.value, appSettings = settings)
            }
        }
        val firstText = passage(0).verseText
        onNodeWithText(firstText).assertIsDisplayed()
        onNodeWithTag("continuous_bible_scrollbar").assertDoesNotExist()
        fun firstLayout(): TextLayoutResult {
            val layouts = mutableListOf<TextLayoutResult>()
            val action = onNodeWithText(firstText).fetchSemanticsNode().config.getOrNull(SemanticsActions.GetTextLayoutResult)
            assertTrue(action?.action?.invoke(layouts) == true)
            return layouts.first()
        }
        val shortStyle = firstLayout().layoutInput.style
        runOnIdle { displayed.value = listOf(combined(40)) }
        waitForIdle()
        onNodeWithText(firstText).assertIsDisplayed()
        onNodeWithTag("continuous_bible_scrollbar").assertIsDisplayed()
        assertEquals(shortStyle.fontSize, firstLayout().layoutInput.style.fontSize)
        assertEquals(shortStyle.fontFamily, firstLayout().layoutInput.style.fontFamily)
        assertEquals(shortStyle.color, firstLayout().layoutInput.style.color)
        System.getProperty("churchPresenter.sidebarReviewDir")?.let { path ->
            val file = File(path, "continuous-bible-top.png")
            file.parentFile.mkdirs()
            ImageIO.write(onRoot().captureToImage().toAwtImage(), "png", file)
        }
        onNodeWithText(passage(39).verseText).performScrollTo().assertIsDisplayed()
        onNodeWithText("启示录 40:1-3").performScrollTo().assertIsDisplayed()
        System.getProperty("churchPresenter.sidebarReviewDir")?.let { path ->
            ImageIO.write(onRoot().captureToImage().toAwtImage(), "png", File(path, "continuous-bible-bottom.png"))
        }
    }

    @Test
    fun `parallel translations remain beside their passage in input order`() = runComposeUiTest {
        val primary = passage(0)
        val secondary = primary.copy(translationFileName = "second.spb", verseText = "The second translation of the first passage")
        val next = passage(1)
        val groups = listOf(listOf(primary, secondary), listOf(next))
        val styles = settings.copy(bibleSettings = settings.bibleSettings.withTranslations(listOf(
            BibleTranslationSettings(fileName = "test.spb", textFontSize = 52),
            BibleTranslationSettings(fileName = "second.spb", textFontSize = 42),
        )))
        setContent {
            Box(Modifier.size(800.dp, 450.dp)) {
                BiblePresenter(selectedVerses = listOf(primary.copy(continuousPassages = groups), secondary.copy(continuousPassages = groups)),
                    appSettings = styles)
            }
        }
        val firstY = onNodeWithText(primary.verseText).fetchSemanticsNode().positionInRoot.y
        val secondY = onNodeWithText(secondary.verseText).fetchSemanticsNode().positionInRoot.y
        val nextY = onNodeWithText(next.verseText).fetchSemanticsNode().positionInRoot.y
        assertTrue(firstY < secondY && secondY < nextY)
    }

    @Test
    fun `continuous passages retain their order when saved and restored`() {
        val original = combined(40)
        val encoded = Json.encodeToString(SelectedVerse.serializer(), original)
        assertEquals(original, Json.decodeFromString(SelectedVerse.serializer(), encoded))
    }
}
