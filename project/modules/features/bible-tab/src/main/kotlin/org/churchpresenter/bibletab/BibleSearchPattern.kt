package org.churchpresenter.bibletab

private val phraseTokens = Regex("""\p{IsHan}|\h+|[^\p{IsHan}\h]+""")
private val hanCharacter = Regex("""\p{IsHan}""")
private val horizontalSpace = Regex("""\h+""")

/** Match spacing used by some Chinese modules without changing the stored verse or display text. */
internal fun bibleSearchPattern(query: String, exact: Boolean): Regex {
    val tokens = phraseTokens.findAll(query.trim()).map { it.value }.toList()
    val phrase = buildString {
        tokens.forEachIndexed { index, token ->
            val previous = tokens.getOrNull(index - 1)
            val next = tokens.getOrNull(index + 1)
            when {
                horizontalSpace.matches(token) && previous?.let(hanCharacter::matches) == true &&
                    next?.let(hanCharacter::matches) == true -> append("""\h*+""")
                else -> {
                    if (previous?.let(hanCharacter::matches) == true && hanCharacter.matches(token)) append("""\h*+""")
                    append(Regex.escape(token))
                }
            }
        }
    }
    return Regex(if (exact) "\\b$phrase\\b" else phrase, RegexOption.IGNORE_CASE)
}
