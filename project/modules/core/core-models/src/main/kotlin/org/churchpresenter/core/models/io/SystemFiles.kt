package org.churchpresenter.core.models.io

/** AppleDouble metadata and Office lock files can carry the original document's extension. */
fun isSystemArtifact(fileName: String): Boolean = fileName.startsWith("._") || fileName.startsWith("~$")

private val systemDirectories = setOf(
    ".spotlight-v100", ".trashes", ".fseventsd", ".temporaryitems", "__macosx", "\$recycle.bin",
)

fun isSystemDirectory(directoryName: String): Boolean = directoryName.lowercase() in systemDirectories
