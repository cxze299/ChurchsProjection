package org.churchpresenter.slides.viewmodel

import kotlinx.coroutines.runBlocking
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals

class PresentationDocumentDiscoveryTest {
    @Test
    fun `discovers documents recursively within the configured root and removes duplicates`() = runBlocking {
        val root = Files.createTempDirectory("presentation-discovery").toFile()
        try {
            val folder = root.resolve("presentations").apply { mkdirs() }
            val nested = folder.resolve("Sunday").apply { mkdirs() }
            val expected = listOf("message.PDF", "slides.pptx", "show.PPS", "show.ppsx", "notes.docx", "legacy.doc").map { name ->
                nested.resolve(name).apply { writeText("document") }
            }
            root.resolve("outside.pdf").writeText("outside configured directory")
            nested.resolve("~\$notes.docx").writeText("office temporary file")
            nested.resolve("._show.PPS").writeText("AppleDouble metadata")
            nested.resolve("notes.txt").writeText("unsupported")
            nested.resolve("node_modules").apply { mkdirs() }.resolve("ignored.pdf").writeText("ignored")
            nested.resolve(".Trashes").apply { mkdirs() }.resolve("removed.pps").writeText("ignored")
            val scan = PresentationDocumentDiscovery.discover(listOf(folder, nested, folder))
            assertEquals(expected.map { it.absolutePath }.toSet(), scan.files.map { it.absolutePath }.toSet())
            assertEquals(expected.size, scan.files.size)
            assertEquals(emptyList(), scan.unreadableDirectories)
        } finally {
            root.deleteRecursively()
        }
    }

    @Test
    fun `missing directories are reported while other configured roots still load`() = runBlocking {
        val root = Files.createTempDirectory("presentation-discovery-missing").toFile()
        try {
            val missing = root.resolve("missing")
            val available = root.resolve("available").apply { mkdir() }
            val hymn = available.resolve("002 Hymn.pps").apply { writeText("document") }
            val scan = PresentationDocumentDiscovery.discover(listOf(missing, available))
            assertEquals(listOf(hymn), scan.files)
            assertEquals(listOf(missing), scan.unreadableDirectories)
        } finally {
            root.deleteRecursively()
        }
    }

    @Test
    fun `a configured mount link is discovered without following links inside it`() = runBlocking {
        val root = Files.createTempDirectory("presentation-discovery-mount")
        val mount = root.resolve("mounted")
        val target = Files.createDirectory(root.resolve("volume"))
        val outside = Files.createDirectory(root.resolve("outside"))
        val nestedLink = target.resolve("linked")
        try {
            val linkCreated = runCatching { Files.createSymbolicLink(mount, target) }.isSuccess
            org.junit.jupiter.api.Assumptions.assumeTrue(linkCreated, "Symbolic links are unavailable")
            Files.createSymbolicLink(nestedLink, outside)
            Files.writeString(target.resolve("002 Hymn.pps"), "document")
            Files.writeString(outside.resolve("003 Out of scope.pps"), "document")
            val scan = PresentationDocumentDiscovery.discover(listOf(mount.toFile()))
            assertEquals(listOf(mount.resolve("002 Hymn.pps").toFile()), scan.files)
            assertEquals(emptyList(), scan.unreadableDirectories)
        } finally {
            Files.deleteIfExists(nestedLink)
            Files.deleteIfExists(mount)
            root.toFile().deleteRecursively()
        }
    }
}
