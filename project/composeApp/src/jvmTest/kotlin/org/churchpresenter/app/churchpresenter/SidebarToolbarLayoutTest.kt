@file:OptIn(androidx.compose.ui.test.ExperimentalTestApi::class)

package org.churchpresenter.app.churchpresenter

import java.util.Locale
import kotlin.test.AfterTest


import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toAwtImage
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.assertHasClickAction
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performMouseInput
import androidx.compose.ui.test.v2.runComposeUiTest
import androidx.compose.ui.unit.dp
import org.churchpresenter.liveoutput.PresenterManager
import org.churchpresenter.settings.AppSettings
import java.io.File
import javax.imageio.ImageIO
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class SidebarToolbarLayoutTest {
    private var originalLocale: Locale = Locale.getDefault()

    @AfterTest
    fun restoreLocale() { Locale.setDefault(originalLocale) }

    @BeforeTest
    fun latch() {
        originalLocale = Locale.getDefault()
        Locale.setDefault(Locale.SIMPLIFIED_CHINESE)
        TestSingletons.latchSkikoHostOs()
        TestSingletons.latchToTestHome()
    }

    private fun checkLayout(width: Int, twoColumns: Boolean) = runComposeUiTest {
        var clears = 0
        val manager = PresenterManager()
        setContent {
            MaterialTheme {
            Box(Modifier.width(width.dp).background(MaterialTheme.colorScheme.surface).testTag("sidebar_toolbar_review")) {
                    SidebarButtons(
                        presenterManager = manager,
                        onClearDisplay = { clears++ },
                        appSettings = AppSettings(),
                        showControl = SidebarShowControl(),
                        onSettingsChange = {},
                        onEditPreviewLayout = {},
                    )
                }
            }
        }
        waitForIdle()
        System.getProperty("churchPresenter.sidebarReviewDir")?.let { directory ->
            val file = File(directory, "toolbar-$width.png")
            file.parentFile.mkdirs()
            ImageIO.write(onNodeWithTag("sidebar_toolbar_review").captureToImage().toAwtImage(), "png", file)
        }
        val output = onNodeWithText("投屏屏幕").assertIsDisplayed().assertHasClickAction().fetchSemanticsNode().boundsInRoot
        val clear = onNodeWithText("清除画面").assertIsDisplayed().assertHasClickAction().fetchSemanticsNode().boundsInRoot
        assertEquals(output.width, clear.width, 1f)
        if (twoColumns) {
            assertEquals(output.top, clear.top, 1f)
            assertTrue(clear.left > output.right)
            assertTrue(onNodeWithText("清除图层").fetchSemanticsNode().boundsInRoot.width > output.width)
        } else {
            assertEquals(output.left, clear.left, 1f)
            assertTrue(clear.top > output.bottom)
        }
        onNodeWithText("清除画面").performClick()
        runOnIdle { assertEquals(1, clears) }
        onNodeWithText("清除画面").performMouseInput { moveTo(center) }
        mainClock.advanceTimeBy(1_000)
        onAllNodesWithText("清除画面").assertCountEquals(1)

    }

    @Test
    fun `narrow sidebar keeps icon and text in one full width click target`() = checkLayout(160, false)

    @Test
    fun `wide sidebar aligns equal width action buttons in two columns`() = checkLayout(280, true)
}
