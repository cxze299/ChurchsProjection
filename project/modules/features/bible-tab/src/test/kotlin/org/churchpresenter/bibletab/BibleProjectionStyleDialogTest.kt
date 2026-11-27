@file:OptIn(androidx.compose.ui.test.ExperimentalTestApi::class)

package org.churchpresenter.bibletab

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.test.hasAnyAncestor
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performMouseInput
import androidx.compose.ui.test.click
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.test.v2.runComposeUiTest
import kotlinx.serialization.json.Json
import org.churchpresenter.settings.AppSettings
import org.churchpresenter.settings.BibleSettings
import org.churchpresenter.settings.BibleTranslationSettings
import org.churchpresenter.settings.OutputProfile
import org.churchpresenter.settings.ScreenAssignment
import org.churchpresenter.settings.profileFor
import org.churchpresenter.settings.resolvedFor
import java.util.Locale
import java.io.File
import javax.imageio.ImageIO
import androidx.compose.ui.graphics.toAwtImage
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class BibleProjectionStyleDialogTest {
    private val originalLocale = Locale.getDefault()
    @BeforeTest fun chinese() { Locale.setDefault(Locale.SIMPLIFIED_CHINESE) }
    @AfterTest fun restoreLocale() { Locale.setDefault(originalLocale) }

    private val settings = AppSettings(bibleSettings = BibleSettings().withTranslations(listOf(
        BibleTranslationSettings(fileName = "test.spb"), BibleTranslationSettings(fileName = "other.spb"),
    )))

    @Test
    fun `the inline style button opens the dialog`() = bibleTab { _, _ ->
        onNodeWithText("样式设置").performClick()
        onNodeWithText("经文投影样式").assertExists()
        System.getProperty("churchPresenter.sidebarReviewDir")?.let { path ->
            val file = File(path, "bible-projection-style-dialog.png")
            file.parentFile.mkdirs()
            ImageIO.write(onNodeWithTag("bible_projection_style_dialog").captureToImage().toAwtImage(), "png", file)
        }
        onNodeWithText("取消").performClick()
        onNodeWithText("经文投影样式").assertDoesNotExist()
    }

    @Test
    fun `dialog previews draft and saves only when applied`() = runComposeUiTest {
        val current = mutableStateOf(settings)
        var dismissed = false
        setContent { MaterialTheme { BibleProjectionStyleDialog(current.value, { change -> current.value = change(current.value) }, { dismissed = true }) } }
        fun replace(tag: String, value: String) {
            onNode(hasSetTextAction() and hasAnyAncestor(hasTestTag(tag))).performScrollTo().performTextReplacement(value)
            waitForIdle()
        }
        onNodeWithTag("bible_projection_text_size_slider").performScrollTo().performMouseInput { click(center) }
        onNodeWithText("应用").assertIsEnabled()
        replace("bible_projection_text_size", "88")
        replace("bible_projection_reference_size", "42")
        replace("bible_projection_line_spacing", "160")
        replace("bible_projection_passage_spacing", "48")
        onNodeWithTag("bible_projection_preview").assertExists()
        assertEquals(settings, current.value)
        onNodeWithText("应用").performClick()
        val projection = current.value.projectionSettings
        val profile = checkNotNull(projection.profileFor(projection.getAssignment(0)))
        val changed = current.value.resolvedFor(profile).bibleSettings
        assertEquals(settings.bibleSettings, current.value.bibleSettings)
        assertEquals(88, changed.translationList().first().textFontSize)
        assertEquals(42, changed.translationList().first().referenceFontSize)
        assertEquals(settings.bibleSettings.translationList()[1], changed.translationList()[1])
        assertEquals(160, changed.continuousLineSpacingPercent)
        assertEquals(48, changed.continuousPassageSpacing)
        assertTrue(dismissed)
        val saved = Json.encodeToString(AppSettings.serializer(), current.value)
        assertEquals(current.value, Json.decodeFromString(AppSettings.serializer(), saved))
    }

    @Test
    fun `cancel discards edited projection settings`() = runComposeUiTest {
        val current = mutableStateOf(settings)
        setContent {
            MaterialTheme { BibleProjectionStyleDialog(current.value, { current.value = it(current.value) }, {}) }
        }
        onNode(hasSetTextAction() and hasAnyAncestor(hasTestTag("bible_projection_text_size")))
            .performScrollTo().performTextReplacement("88")
        onNodeWithText("取消").performClick()
        assertEquals(settings, current.value)
    }

    @Test
    fun `font changes affect only the chosen translation and preserve other styling`() {
        val changed = updateBibleProjectionTranslation(settings.bibleSettings, "other.spb", "textFontType") { it.copy(textFontType = "Microsoft YaHei") }
        assertEquals(settings.bibleSettings.translationList().first(), changed.translationList().first())
        assertEquals("Microsoft YaHei", changed.translationList()[1].textFontType)
        assertTrue("textFontType" in changed.translationList()[1].ownStyleKeys)
    }

    @Test
    fun `selected display uses slot zero and keeps unrelated output profiles unchanged`() {
        val screen = OutputProfile(id = "screen", bibleSettings = settings.bibleSettings.updateTranslation(0) {
            it.copy(textFontSize = 120, textColor = "#ABCDEF")
        })
        val other = OutputProfile(id = "other", bibleSettings = settings.bibleSettings)
        val before = settings.copy(projectionSettings = settings.projectionSettings.copy(
            outputProfiles = listOf(other, screen),
            screenAssignments = listOf(ScreenAssignment(activeProfileId = screen.id), ScreenAssignment(activeProfileId = other.id)),
            quickPlayDisplay = 1,
        ))
        val after = before.applyBibleProjectionStyle { bible ->
            updateBibleProjectionTranslation(bible, "test.spb", "textFontSize") { it.copy(textFontSize = 88) }
        }
        val profile = checkNotNull(after.projectionSettings.profileFor(after.projectionSettings.getAssignment(0)))
        val rendered = after.resolvedFor(profile).bibleSettings.translationList().first()
        assertEquals(88, rendered.textFontSize)
        assertEquals("#ABCDEF", rendered.textColor)
        assertEquals(other, after.projectionSettings.outputProfiles.first())
    }
}
