package org.churchpresenter.slides.viewmodel

import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals

class HymnQuickPlayTest {
    private val root = File("hymns").absoluteFile
    private val files = listOf(
        "BookA/001 Amazing Grace.pptx", "BookA/001 Amazing Grace.pdf",
        "BookA/012 Other Hymn.pptx", "BookB/001 Different Hymn.pdf",
    ).map(root::resolve)

    @Test
    fun `number lookup respects book and preferred format without matching longer numbers`() {
        assertEquals(listOf(files[0], files[3]), HymnQuickPlay.find(files, root, "1", "BookA", "pptx"))
        assertEquals(listOf(files[1], files[3]), HymnQuickPlay.find(files, root, "001", "BookA", "pdf"))
        assertEquals(listOf(files[2]), HymnQuickPlay.find(files, root, "12", "BookA", "pptx"))
    }

    @Test
    fun `empty search lists only the default book using the preferred file format`() {
        assertEquals(listOf(files[1], files[2]), HymnQuickPlay.find(files, root, "", "BookA", "pdf"))
    }

    @Test
    fun `title lookup is case insensitive and missing preferred formats fall back`() {
        assertEquals(listOf(files[0]), HymnQuickPlay.find(files, root, "amazing", "BookA", "pptx"))
        assertEquals(listOf(files[3], files[0]), HymnQuickPlay.find(files, root, "1", "BookB", "pptx"))
        assertEquals(3, HymnQuickPlay.find(files, root, " ", "", "pdf").size)
        assertEquals(2, HymnQuickPlay.find(files, root, "1", "", "pdf").size)
    }

    @Test
    fun `format qualifiers work with numbers titles and pinyin initials across hymn books`() {
        val pps = root.resolve("BookB/002 奇异恩典.PPS")
        val ppsx = root.resolve("BookB/002 奇异恩典.ppsx")
        val library = files + listOf(pps, ppsx)
        for (query in listOf(".pps", "*.PPS", "002.pps", "2 PPS", "奇异恩典.pps", "qyed .pps")) {
            assertEquals(listOf(pps), HymnQuickPlay.find(library, root, query, "BookA", "pdf"), query)
        }
        assertEquals(listOf(ppsx), HymnQuickPlay.find(library, root, "2", "BookA", "ppsx"))
        assertEquals(listOf(pps), HymnQuickPlay.find(library, root, "2", "BookA", "pps"))
    }

    @Test
    fun `mounted roots are searched independently and unicode names match in either form`() {
        val shared = File("shared").absoluteFile
        val folder = shared.resolve("BookA")
        val mounted = folder.resolve("001 Amazing Grace.pptx")
        val direct = shared.resolve("002 Cafe\u0301.pps")
        val roots = listOf(root, shared, folder)
        val library = files + listOf(mounted, direct)
        assertEquals("BookA", HymnQuickPlay.book(mounted, root, roots))
        assertEquals("shared", HymnQuickPlay.book(direct, root, roots))
        assertEquals(listOf(files[0], mounted),
            HymnQuickPlay.find(library, root, "Amazing Grace", "BookA", "pptx", roots))
        assertEquals(listOf(direct), HymnQuickPlay.find(library, root, "Café", "Old book", "pptx", roots))
        assertEquals(5, HymnQuickPlay.find(library, root, "", "Old book", "pptx", roots).size)
    }
}
