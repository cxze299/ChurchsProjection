package org.churchpresenter.sharedui.models

import androidx.compose.ui.input.key.Key
import org.churchpresenter.core.models.shortcuts.KeyChord

internal fun primaryShortcut(
    key: Key,
    shift: Boolean = false,
    mac: Boolean = System.getProperty("os.name", "").startsWith("Mac", ignoreCase = true),
): KeyChord = KeyChord.of(key, ctrl = !mac, meta = mac, shift = shift)
