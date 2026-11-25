package org.churchpresenter.bibletab

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import org.churchpresenter.bible.SpbFixture
import org.churchpresenter.settings.AppSettings
import org.churchpresenter.settings.BibleSettings
import org.churchpresenter.core.models.bible.SelectedVerse
import java.io.File
import java.nio.file.Files
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class BibleQuickPassagesTest {
    private lateinit var directory: File
    private lateinit var vm: BibleViewModel

    @BeforeTest
    fun loadBible() {
        directory = Files.createTempDirectory("cp-quick-passages").toFile()
        SpbFixture.spbFile(directory, content = SpbFixture.buildContent(
            "Test Bible",
            listOf(SpbFixture.Book(40, "Matthew", 5), SpbFixture.Book(66, "Revelation", 6)),
            (1..12).map { SpbFixture.Verse(40, 5, it, "Matthew text $it") } +
                (1..8).map { SpbFixture.Verse(66, 6, it, "Revelation text $it") },
        ))
        vm = BibleViewModel(AppSettings(bibleSettings = BibleSettings(storageDirectory = directory.absolutePath, primaryBible = "test.spb")),
            dispatcher = Dispatchers.Unconfined, ioDispatcher = Dispatchers.Unconfined)
        runBlocking { vm.awaitFullyLoaded() }
    }

    @AfterTest
    fun dispose() {
        vm.dispose()
        directory.deleteRecursively()
    }

    @Test
    fun `both separator forms resolve the complete reversed range and the next book`() = runBlocking {
        val attached = assertNotNull(vm.quickPassageReferences("mtfy 5 12 5qsl 6 7 8"))
        assertEquals(attached, vm.quickPassageReferences("mtfy 5 12 5 qsl 6 7 8"))
        assertEquals(5, attached.first().verseStart)
        assertEquals(12, attached.first().verseEnd)
        val passages = assertNotNull(vm.resolveQuickPassages(attached))
        assertEquals(listOf("Matthew", "Revelation"), passages.map { it.first().bookName })
        assertEquals("5-12", passages.first().first().verseRange)
        assertEquals("7-8", passages.last().first().verseRange)
        assertTrue(passages.first().first().verseText.contains("Matthew text 12"))
        assertTrue(passages.last().first().verseText.contains("Revelation text 8"))
    }

    @Test
    fun `references have no fixed passage count limit`() = runBlocking {
        val query = List(128) { if (it % 2 == 0) "mtfy 5 5 12" else "qsl 6 7 8" }.joinToString(" ")
        val refs = assertNotNull(vm.quickPassageReferences(query))
        assertEquals(128, refs.size)
        assertEquals(128, assertNotNull(vm.resolveQuickPassages(refs)).size)
    }

    @Test
    fun `incomplete unknown and invalid segments never produce a partial request`() = runBlocking {
        listOf("mtfy 5 12 5qsl", "mtfy 5 12 5 qsl 6", "mtfy 5 12 5 xyz 6 7 8", "mtfy 0 1 2", "mtfy 5 0 2")
            .forEach { assertNull(vm.quickPassageReferences(it), it) }
        val refs = assertNotNull(vm.quickPassageReferences("mtfy 5 5 12qsl 6 7 99"))
        val before = vm.selectedBookIndex.value
        assertNull(vm.resolveQuickPassages(refs))
        assertEquals(before, vm.selectedBookIndex.value)
    }

    @Test
    fun `the live label describes every passage and accepts uppercase initials`() {
        assertEquals("Matthew 5章5-12节；Revelation 6章7-8节", vm.quickPassageLabel("MTFY 5 12 5QSL 6 7 8"))
    }

    @Test
    fun `a range replaces its earlier single verse and retains another book`() {
        val single = SelectedVerse(bookName = "Matthew", chapter = 5, verseNumber = 5, verseText = "single")
        val other = SelectedVerse(bookName = "Revelation", chapter = 6, verseNumber = 7, verseText = "other")
        val range = single.copy(verseRange = "5-12", verseText = "range")
        assertEquals(listOf(listOf(other), listOf(range)), appendPassage(listOf(listOf(single), listOf(other)), listOf(range)))
    }
}
