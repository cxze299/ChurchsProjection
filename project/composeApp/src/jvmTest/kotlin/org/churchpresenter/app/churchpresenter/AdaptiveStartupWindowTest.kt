package org.churchpresenter.app.churchpresenter

import org.churchpresenter.settings.AppSettings
import org.churchpresenter.settings.DisplayRect
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class AdaptiveStartupWindowTest {
    @Test
    fun `startup opens ordinary placement with both layouts collapsed`() {
        val saved = AppSettings(windowPlacement = "fullscreen", windowWidth = 3840, windowHeight = 2160)
        val startup = startupOperatorSettings(saved)
        assertEquals("floating", startup.windowPlacement)
        assertTrue(startup.windowedLayout.schedulePanelCollapsed)
        assertTrue(startup.windowedLayout.previewPanelCollapsed)
        assertTrue(startup.maximizedLayout.schedulePanelCollapsed)
        assertTrue(startup.maximizedLayout.previewPanelCollapsed)
        assertEquals(saved.bibleSettings, startup.bibleSettings)
        assertEquals(saved.projectionSettings, startup.projectionSettings)
        assertEquals(saved.windowedLayout.schedulePanelWidthDp, startup.windowedLayout.schedulePanelWidthDp)
    }

    @Test
    fun `ordinary displays get a centered compact window`() {
        assertEquals(DisplayRect(320, 140, 1280, 800), adaptiveMainWindowBounds(DisplayRect(0, 20, 1920, 1040)))
        assertEquals(DisplayRect(137, 36, 1092, 600), adaptiveMainWindowBounds(DisplayRect(0, 8, 1366, 656)))
    }

    @Test
    fun `small and scaled screens keep the complete frame inside their usable area`() {
        listOf(DisplayRect(0, 0, 640, 440), DisplayRect(-1280, 36, 1280, 684),
            DisplayRect(1920, -1080, 1920, 1040)).forEach { area ->
            val window = adaptiveMainWindowBounds(area, minimumWidth = 1600, minimumHeight = 1200)
            assertTrue(window.x >= area.x + 16)
            assertTrue(window.y >= area.y + 16)
            assertTrue(window.x + window.width <= area.x + area.width - 16)
            assertTrue(window.y + window.height <= area.y + area.height - 16)
        }
    }

    @Test
    fun `large monitors keep a bounded default size`() {
        assertEquals(1280, adaptiveMainWindowBounds(DisplayRect(0, 0, 3840, 2080)).width)
        assertEquals(800, adaptiveMainWindowBounds(DisplayRect(0, 0, 3840, 2080)).height)
    }
}
