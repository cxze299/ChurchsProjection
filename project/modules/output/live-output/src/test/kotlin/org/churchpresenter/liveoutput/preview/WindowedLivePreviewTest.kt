@file:OptIn(androidx.compose.ui.test.ExperimentalTestApi::class)

package org.churchpresenter.liveoutput.preview

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.v2.runComposeUiTest
import androidx.compose.ui.unit.dp
import androidx.compose.ui.graphics.toAwtImage
import org.churchpresenter.core.models.bible.SelectedVerse
import org.churchpresenter.core.models.songs.LyricSection
import org.churchpresenter.liveoutput.PresenterManager
import org.churchpresenter.settings.AppSettings
import org.churchpresenter.settings.OutputProfile
import org.churchpresenter.settings.ProjectionSettings
import org.churchpresenter.settings.ScreenAssignment
import org.churchpresenter.settings.utils.Constants
import org.churchpresenter.sharedui.models.Presenting
import java.io.File
import java.util.Locale
import javax.imageio.ImageIO
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test

class WindowedLivePreviewTest {
    private val originalLocale = Locale.getDefault()
    @BeforeTest fun english() { Locale.setDefault(Locale.ENGLISH) }
    @AfterTest fun restoreLocale() { Locale.setDefault(originalLocale) }

    private val settings = AppSettings(projectionSettings = ProjectionSettings(
        windowedProjection = true,
        devWindowCount = 3,
        outputProfiles = listOf(OutputProfile(id = "main", displayMode = Constants.DISPLAY_MODE_FULLSCREEN)),
        screenAssignments = listOf(ScreenAssignment(activeProfileId = "main", targetDisplay = Constants.KEY_TARGET_NONE)),
    ))

    @Test
    fun `a release independent window previews live scripture and follows content changes`() = runComposeUiTest {
        val manager = PresenterManager().apply {
            setPresentingMode(Presenting.BIBLE)
            setDisplayedVerses(listOf(SelectedVerse(bookName = "Genesis", chapter = 1,
                verseNumber = 3, verseText = "Let there be light")))
        }
        setContent {
            MaterialTheme {
                Box(Modifier.width(280.dp).testTag("windowed_preview")) {
                    LivePreviewPanel(manager, settings, isRelease = true)
                }
            }
        }
        onNodeWithText("Screen 1").assertExists()
        onNodeWithText("Screen 2").assertDoesNotExist()
        onNodeWithText("Let there be light", substring = true).assertExists()
        System.getProperty("churchPresenter.sidebarReviewDir")?.let { directory ->
            val file = File(directory, "windowed-live-preview.png")
            file.parentFile.mkdirs()
            ImageIO.write(onNodeWithTag("windowed_preview").captureToImage().toAwtImage(), "png", file)
        }
        runOnIdle {
            manager.setDisplayedVerses(listOf(SelectedVerse(bookName = "Genesis", chapter = 1,
                verseNumber = 4, verseText = "God saw that the light was good")))
        }
        onNodeWithText("God saw that the light was good", substring = true).assertExists()
        onNodeWithText("Let there be light", substring = true).assertDoesNotExist()
        runOnIdle {
            manager.setDisplayedLyricSection(LyricSection(type = "verse", title = "Grace", lines = listOf("Amazing grace")))
            manager.setPresentingMode(Presenting.LYRICS)
        }
        onNodeWithText("Amazing grace", substring = true).assertExists()
    }

    @Test
    fun `switching window output on and off adds and removes the release preview`() = runComposeUiTest {
        val current = mutableStateOf(settings.copy(projectionSettings = settings.projectionSettings.copy(windowedProjection = false)))
        val manager = PresenterManager()
        setContent { MaterialTheme { LivePreviewPanel(manager, current.value, isRelease = true) } }
        onNodeWithText("Screen 1").assertDoesNotExist()
        runOnIdle { current.value = settings }
        onNodeWithText("Screen 1").assertExists()
        runOnIdle { current.value = current.value.copy(projectionSettings = current.value.projectionSettings.copy(windowedProjection = false)) }
        onNodeWithText("Screen 1").assertDoesNotExist()
    }

    @Test
    fun `an explicitly selected monitor is previewed even when the stored assignment is none`() = runComposeUiTest {
        val app = settings.copy(projectionSettings = settings.projectionSettings.copy(windowedProjection = false, quickPlayDisplay = 0))
        val manager = PresenterManager()
        setContent {
            MaterialTheme {
                val entries = previewEntries(app.projectionSettings, 1, 1, false,
                    manager, app, "", "", null, {}, quickDisplay = 0)
                entries.forEach { it.content(Modifier.width(280.dp), false) }
            }
        }
        onNodeWithText("Screen 1").assertExists()
        onNodeWithText("Screen 2").assertDoesNotExist()
    }
}
