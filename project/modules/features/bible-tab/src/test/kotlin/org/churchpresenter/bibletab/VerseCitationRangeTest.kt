package org.churchpresenter.bibletab

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import org.churchpresenter.bible.SpbFixture
import org.churchpresenter.settings.AppSettings
import org.churchpresenter.settings.BibleSettings
import java.io.File
import java.nio.file.Files
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class VerseCitationRangeTest {
    private lateinit var directory: File
    private lateinit var vm: BibleViewModel

    @BeforeTest
    fun loadChapter(): Unit = runBlocking {
        directory = Files.createTempDirectory("cp-citation-range").toFile()
        SpbFixture.spbFile(directory, content = SpbFixture.buildContent("Test Bible",
            listOf(SpbFixture.Book(10, "撒母耳记下", 8)),
            (5..18).filter { it != 12 }.map { SpbFixture.Verse(10, 8, it, "经文 $it") }))
        vm = BibleViewModel(AppSettings(bibleSettings = BibleSettings(
            storageDirectory = directory.absolutePath, primaryBible = "test.spb")),
            dispatcher = Dispatchers.Unconfined, ioDispatcher = Dispatchers.Unconfined)
        vm.resolveVerseSelection("撒母耳记下", 8, 5)
    }

    @AfterTest
    fun dispose() {
        vm.dispose()
        directory.deleteRecursively()
    }

    @Test
    fun `a continuous selection displays its endpoints when the module skips a number`() {
        vm.selectVerse(0)
        vm.shiftClickVerse(vm.verses.value.lastIndex)
        val shown = vm.getSelectedVerses().first()
        assertEquals("5-18", shown.verseRange)
        assertTrue(shown.verseText.contains("经文 18"))
    }

    @Test
    fun `separated selections retain their gaps as compact ranges`() {
        vm.selectVerse(0)
        listOf(1, 2, 8, 9).forEach(vm::ctrlClickVerse)
        assertEquals("5-7,14-15", vm.getSelectedVerses().first().verseRange)
    }
}
