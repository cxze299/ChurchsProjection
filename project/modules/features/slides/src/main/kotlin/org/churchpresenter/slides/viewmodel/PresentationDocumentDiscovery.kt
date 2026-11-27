package org.churchpresenter.slides.viewmodel

import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import org.churchpresenter.presentationengine.PresentationLoader
import java.io.File
import java.io.IOException
import java.nio.file.FileVisitResult
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.SimpleFileVisitor
import java.nio.file.attribute.BasicFileAttributes

internal object PresentationDocumentDiscovery {
    private val ignoredDirectories = setOf(
        "node_modules", "dist", "build", ".git", "target", "\$recycle.bin",
        ".spotlight-v100", ".trashes", ".fseventsd", ".temporaryitems",
    )

    data class Result(val files: List<File>, val unreadableDirectories: List<File>)

    suspend fun discover(directories: List<File>): Result {
        val context = currentCoroutineContext()
        val found = linkedMapOf<Path, File>()
        val unreadable = linkedSetOf<File>()
        directories.map { it.toPath().toAbsolutePath().normalize() }.distinct()
            .sortedByDescending { it.nameCount }.forEach { root ->
            context.ensureActive()
            try {
                // Follow a link chosen as a root, while confining the scan to that root.
                val resolved = root.toRealPath()
                if (!Files.isDirectory(resolved)) {
                    unreadable += root.toFile()
                    return@forEach
                }
                fun configuredPath(path: Path): File = root.resolve(resolved.relativize(path)).toFile()
                fun ignored(path: Path): Boolean =
                    path != resolved && path.fileName?.toString()?.lowercase() in ignoredDirectories
                Files.walkFileTree(resolved, object : SimpleFileVisitor<Path>() {
                    override fun preVisitDirectory(dir: Path, attrs: BasicFileAttributes): FileVisitResult {
                        context.ensureActive()
                        return if (ignored(dir)) {
                            FileVisitResult.SKIP_SUBTREE
                        } else {
                            FileVisitResult.CONTINUE
                        }
                    }

                    override fun visitFile(file: Path, attrs: BasicFileAttributes): FileVisitResult {
                        context.ensureActive()
                        val document = file.toFile()
                        if (attrs.isRegularFile && !document.name.startsWith("~\$") && !document.name.startsWith("._") &&
                            document.extension.lowercase() in PresentationLoader.SUPPORTED_EXTENSIONS
                        ) {
                            found.putIfAbsent(file, configuredPath(file))
                        }
                        return FileVisitResult.CONTINUE
                    }

                    override fun visitFileFailed(file: Path, exc: IOException): FileVisitResult {
                        context.ensureActive()
                        if (!ignored(file)) unreadable += configuredPath(file)
                        return FileVisitResult.CONTINUE
                    }

                    override fun postVisitDirectory(dir: Path, exc: IOException?): FileVisitResult {
                        context.ensureActive()
                        if (exc != null) unreadable += configuredPath(dir)
                        return FileVisitResult.CONTINUE
                    }
                })
            } catch (_: IOException) {
                unreadable += root.toFile()
            } catch (_: SecurityException) {
                unreadable += root.toFile()
            }
        }
        return Result(
            found.values.sortedWith(compareByDescending<File> { it.lastModified() }.thenBy { it.absolutePath }),
            unreadable.toList(),
        )
    }
}
