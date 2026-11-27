package org.churchpresenter.settings

import kotlinx.serialization.Serializable

@Serializable
enum class BibleCitationStyle(val prefix: String, val suffix: String) {
    PLAIN("", ""),
    DOUBLE_BRACKETS("〖", "〗"),
    CORNER_BRACKETS("「", "」"),
    SQUARE_BRACKETS("[", "]");

    fun format(reference: String): String = "$prefix$reference$suffix"
}
