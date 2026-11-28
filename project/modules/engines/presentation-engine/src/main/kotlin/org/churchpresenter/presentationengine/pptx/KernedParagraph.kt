/*
 * The line breaking below is a port of Apache POI 5.5.1's DrawTextParagraph.breakText,
 * getAttributedString, processGlyphs and nextPart, licensed under the Apache License, Version 2.0
 * (http://www.apache.org/licenses/LICENSE-2.0). Changed: each run's font has kerning turned on, a
 * line's trailing spaces may hang past the wrapping width, per-deck embedded subset fonts are
 * honoured directly, PUA symbol codepoints (0xF000..0xF0FF) are probed for musical notation fonts,
 * normAutofit scaling and HSLF/XSLF character spacing are applied, and explicit tab stops (\t)
 * advance to PowerPoint's horizontal tab positions.
 */
package org.churchpresenter.presentationengine.pptx

import org.apache.poi.common.usermodel.fonts.FontGroup
import org.apache.poi.common.usermodel.fonts.FontInfo
import org.apache.poi.hslf.model.textproperties.TextProp
import org.apache.poi.hslf.usermodel.HSLFTextRun
import org.apache.poi.sl.draw.DrawFactory
import org.apache.poi.sl.draw.DrawFontManager
import org.apache.poi.sl.draw.DrawPaint
import org.apache.poi.sl.draw.DrawTextFragment
import org.apache.poi.sl.draw.DrawTextParagraph
import org.apache.poi.sl.usermodel.HighlightColorSupport
import org.apache.poi.sl.usermodel.TextParagraph
import org.apache.poi.sl.usermodel.TextParagraph.TextAlign
import org.apache.poi.sl.usermodel.TextRun
import org.apache.poi.sl.usermodel.TextShape
import org.apache.poi.util.Units
import org.apache.poi.xslf.usermodel.XSLFTextParagraph
import org.apache.poi.xslf.usermodel.XSLFTextRun
import org.apache.poi.xslf.usermodel.XSLFTextShape
import org.openxmlformats.schemas.presentationml.x2006.main.CTShape
import java.awt.Font
import java.awt.Graphics2D
import java.awt.font.GraphicAttribute
import java.awt.font.LineBreakMeasurer
import java.awt.font.TextAttribute
import java.awt.font.TextLayout
import java.awt.geom.Rectangle2D
import java.text.AttributedCharacterIterator
import java.text.AttributedCharacterIterator.Attribute
import java.text.AttributedString

/** A laid-out line and the index in the paragraph's text where it ends. */
private class Line(val layout: TextLayout, val end: Int)

/** One attribute over a range of the paragraph's text — POI's private `AttributedStringData`. */
private class RangeAttribute(val attribute: Attribute, val value: Any, val begin: Int, val end: Int)

/**
 * Zero-height horizontal spacer representing a single `\t` advanced to its PowerPoint tab stop.
 */
private class TabGraphicAttribute(private val advanceWidth: Float) : GraphicAttribute(ROMAN_BASELINE) {
    override fun getAscent(): Float = 0f
    override fun getDescent(): Float = 0f
    override fun getAdvance(): Float = advanceWidth.coerceAtLeast(1f)
    override fun getBounds(): Rectangle2D = Rectangle2D.Float(0f, 0f, advance, 0f)
    override fun draw(graphics: Graphics2D, x: Float, y: Float) = Unit
}

/**
 * A paragraph drawable that lays its lines out with kerning, reports the descent of its last
 * line, and opens the two steps [stackParagraphs] calls from outside POI's package.
 */
internal class KernedParagraph(paragraph: TextParagraph<*, *, *>) : DrawTextParagraph(paragraph) {
    val isSingleLine: Boolean get() = lines.size == 1

    val lastLineDescent: Double
        get() = lines.lastOrNull()?.layout?.descent?.toDouble() ?: 0.0

    fun layOut(graphics: Graphics2D) = breakText(graphics)

    fun markFirst(first: Boolean) = setFirstParagraph(first)

    override fun breakText(graphics: Graphics2D) {
        lines.clear()
        val factory = DrawFactory.getInstance(graphics)
        val text = StringBuilder()
        val attributes = attributedRuns(graphics, text)
        val plain = text.toString()
        val withBreaks = AttributedString(plain)
        val withoutBreaks = AttributedString(plain.replace(LINE_BREAKS, " "))
        for (range in attributes) {
            withBreaks.addAttribute(range.attribute, range.value, range.begin, range.end)
            withoutBreaks.addAttribute(range.attribute, range.value, range.begin, range.end)
        }
        val iterator = withBreaks.iterator
        val iteratorNoBreaks = withoutBreaks.iterator
        val measurer = LineBreakMeasurer(iterator, graphics.fontRenderContext)
        val largestSize = attributes.filter { it.attribute == TextAttribute.SIZE }
            .maxOfOrNull { (it.value as Number).toFloat() } ?: 0f
        var done = false
        while (!done) {
            val start = measurer.position
            // POI adds a point to absorb rounding; a box narrower than its insets still gets one.
            val wrappingWidth = (getWrappingWidth(lines.isEmpty(), graphics) + 1).coerceAtLeast(1.0).toFloat()
            val line = nextLine(measurer, plain, wrappingWidth, largestSize)
            if (line != null) {
                lines.add(factory.getTextFragment(line.layout, AttributedString(iteratorNoBreaks, start, line.end)))
            }
            done = line == null || line.end == iterator.endIndex
        }
        rawText = plain
    }

    /** The next line from [measurer]'s position, or null when not even part of a word fits. */
    private fun nextLine(measurer: LineBreakMeasurer, text: String, width: Float, largestSize: Float): Line? {
        // A paragraph opening with a line break gets an empty first line.
        if (measurer.position == 0 && text.startsWith("\n")) {
            return measurer.nextLayout(width, 1, false)?.let { Line(it, 1) }
        }
        val start = measurer.position
        val nextBreak = text.indexOf("\n", start + 1).takeIf { it != -1 } ?: text.length
        val layout = measurer.nextHangingLayout(text, width, nextBreak, largestSize)
            // A word wider than the whole line comes back null; break inside it instead.
            ?: measurer.nextLayout(width, nextBreak, false)
            ?: return null
        val end = measurer.position
        // The break character itself is drawn on neither line.
        if (end < text.length && text[end] == '\n') measurer.position = end + 1
        val align = paragraph.textAlign
        val justified = align == TextAlign.JUSTIFY || align == TextAlign.JUSTIFY_LOW
        return Line(if (justified) layout.getJustifiedLayout(width) else layout, end)
    }

    private fun attributedRuns(graphics: Graphics2D, text: StringBuilder): List<RangeAttribute> {
        val paint = DrawPaint(paragraph.parentShape)
        val fonts = DrawFactory.getInstance(graphics).getFontManager(graphics)
        val deckFonts = deckEmbeddedFonts(graphics)
        val fontScale = normAutofitFontScale(paragraph.parentShape)
        val tabStops = explicitTabStopsPt(paragraph)
        val defaultTabPt = (paragraph.defaultTabSize ?: DEFAULT_TAB_SIZE_PT).coerceAtLeast(1.0)
        val ranges = mutableListOf<RangeAttribute>()
        var lineAdvanceX = 0.0
        for (run in paragraph) {
            val raw = run.rawText.orEmpty()
            val hasTab = '\t' in raw
            var runText = if (hasTab) renderableTextPreservingTabs(run) else getRenderableText(graphics, run)
            if (runText.isEmpty()) continue
            val fontInfo = run.getFontInfo(null)
            runText = fonts.mapFontCharset(graphics, fontInfo, runText)
            val att = runAttributes(graphics, paint, run, fontScale)
            val runFont = fontInfo ?: PlainFontInfo(paragraph.defaultFontFamily)
            val mappedName = mappedFamily(graphics, fonts, runFont)
            att[TextAttribute.FAMILY] = mappedName
            att[TextAttribute.KERNING] = TextAttribute.KERNING_ON
            att[TextAttribute.FONT] = buildAwtFont(deckFonts, mappedName, att)
            val begin = text.length
            val resolved = partFontsAndRemap(graphics, fonts, deckFonts, run, runText, begin, att)
            val finalRunText = resolved.text
            text.append(finalRunText)
            att.forEach { (key, value) -> ranges += RangeAttribute(key, value, begin, text.length) }
            textEffectsOf(run)?.let { ranges += RangeAttribute(TextEffectsKey, it, begin, text.length) }
            ranges += resolved.ranges
            if (hasTab) {
                lineAdvanceX = applyTabStops(
                    graphics = graphics,
                    runText = finalRunText,
                    begin = begin,
                    baseFont = att[TextAttribute.FONT] as Font,
                    startX = lineAdvanceX,
                    tabStops = tabStops,
                    defaultTabPt = defaultTabPt,
                    ranges = ranges,
                )
            } else {
                val lastBreak = finalRunText.lastIndexOf('\n')
                val tail = if (lastBreak >= 0) finalRunText.substring(lastBreak + 1) else finalRunText
                if (lastBreak >= 0) lineAdvanceX = 0.0
                if (tail.isNotEmpty()) {
                    val font = att[TextAttribute.FONT] as Font
                    lineAdvanceX += TextLayout(tail, font, graphics.fontRenderContext).advance.toDouble()
                }
            }
        }
        // A paragraph needs at least one character to be measured at all.
        if (text.isEmpty()) {
            text.append(" ")
            val defaultName = mappedFamily(graphics, fonts, PlainFontInfo(paragraph.defaultFontFamily))
            val att = mutableMapOf<Attribute, Any>(
                TextAttribute.SIZE to (paragraph.defaultFontSize * fontScale).toFloat(),
                TextAttribute.FAMILY to defaultName,
                TextAttribute.KERNING to TextAttribute.KERNING_ON,
            )
            att[TextAttribute.FONT] = buildAwtFont(deckFonts, defaultName, att)
            att.forEach { (key, value) -> ranges += RangeAttribute(key, value, 0, 1) }
        }
        return ranges
    }

    /**
     * POI's `getBullet`, with two changes: the style comes from [bulletStyleOf], so a numbered
     * paragraph gets its number, and the bullet carries the outline, glow and shadow of the text it
     * leads, as PowerPoint draws it.
     */
    override fun getBullet(graphics: Graphics2D, firstLineAttr: AttributedCharacterIterator): DrawTextFragment? {
        val style = bulletStyleOf(paragraph) ?: return null
        val numbering = style.autoNumberingScheme
        val character = (if (numbering != null) numbering.format(autoNbrIdx) else style.bulletCharacter)
            ?: return null
        val fonts = DrawFactory.getInstance(graphics).getFontManager(graphics)
        val deckFonts = deckEmbeddedFonts(graphics)
        val named = PlainFontInfo(style.bulletFont ?: paragraph.defaultFontFamily)
        val font = PlainFontInfo(mappedFamily(graphics, fonts, named))
        val textSize = (firstLineAttr.getAttribute(TextAttribute.SIZE) as Number).toDouble()
        // OOXML bullet size: positive is a percentage of the text's, negative is absolute points.
        val relative = style.bulletFontSize ?: PERCENT
        val size = if (relative > 0) textSize * relative / PERCENT else -relative
        val att = mutableMapOf<Attribute, Any>(
            TextAttribute.FAMILY to font.typeface,
            TextAttribute.SIZE to size.toFloat(),
        )
        val color = style.bulletFontColor
            ?.let { DrawPaint(paragraph.parentShape).getPaint(graphics, it) }
            ?: firstLineAttr.getAttribute(TextAttribute.FOREGROUND)
        color?.let { att[TextAttribute.FOREGROUND] = it }
        att[TextAttribute.FONT] = buildAwtFont(deckFonts, font.typeface, att)
        firstLineAttr.getAttribute(TextEffectsKey)?.let { att[TextEffectsKey] = it }

        val text = AttributedString(fonts.mapFontCharset(graphics, font, character))
        att.forEach { (key, value) -> text.addAttribute(key, value) }
        val layout = TextLayout(text.iterator, graphics.fontRenderContext)
        return DrawFactory.getInstance(graphics).getTextFragment(layout, text)
    }

    private fun mappedFamily(graphics: Graphics2D, fonts: DrawFontManager, font: FontInfo): String =
        fonts.getMappedFont(graphics, font)?.typeface ?: paragraph.defaultFontFamily

    private fun runAttributes(
        graphics: Graphics2D,
        paint: DrawPaint,
        run: TextRun,
        fontScale: Double,
    ): MutableMap<Attribute, Any> {
        val att = mutableMapOf<Attribute, Any>()
        paint.getPaint(graphics, run.fontColor)?.let { att[TextAttribute.FOREGROUND] = it }
        if (run is HighlightColorSupport) {
            run.highlightColor
                ?.let { style -> paint.getPaint(graphics, style) }
                ?.let { att[TextAttribute.BACKGROUND] = it }
        }
        val fontSize = ((run.fontSize ?: paragraph.defaultFontSize) * fontScale).toFloat()
        att[TextAttribute.SIZE] = fontSize
        // PowerPoint stores character spacing in points; AWT tracking uses a fraction of the font size.
        val spacing = characterSpacingPt(run) * fontScale
        if (spacing.isFinite() && spacing != 0.0 && fontSize > 0f) {
            att[TextAttribute.TRACKING] = (spacing / fontSize).toFloat()
        }
        if (run.isBold) att[TextAttribute.WEIGHT] = TextAttribute.WEIGHT_BOLD
        if (run.isItalic) att[TextAttribute.POSTURE] = TextAttribute.POSTURE_OBLIQUE
        if (run.isUnderlined) {
            att[TextAttribute.UNDERLINE] = TextAttribute.UNDERLINE_ON
            att[TextAttribute.INPUT_METHOD_UNDERLINE] = TextAttribute.UNDERLINE_LOW_TWO_PIXEL
        }
        if (run.isStrikethrough) att[TextAttribute.STRIKETHROUGH] = TextAttribute.STRIKETHROUGH_ON
        if (run.isSubscript) att[TextAttribute.SUPERSCRIPT] = TextAttribute.SUPERSCRIPT_SUB
        if (run.isSuperscript) att[TextAttribute.SUPERSCRIPT] = TextAttribute.SUPERSCRIPT_SUPER
        run.hyperlink?.let { link ->
            link.address?.let { att[HYPERLINK_HREF] = it }
            link.label?.let { att[HYPERLINK_LABEL] = it }
        }
        return att
    }

    private class ResolvedRunText(val text: String, val ranges: List<RangeAttribute>)

    /**
     * Resolves per-font-group faces (Latin, East Asian, Symbol, ...) while preferring per-deck
     * embedded fonts and probing Windows Symbol PUA codepoints (`0xF000..0xF0FF`) so custom
     * musical notation fonts never fall back to `SansSerif`.
     */
    private fun partFontsAndRemap(
        graphics: Graphics2D,
        fonts: DrawFontManager,
        deckFonts: Map<String, List<Font>>,
        run: TextRun,
        rawText: String,
        begin: Int,
        base: Map<Attribute, Any>,
    ): ResolvedRunText {
        val ranges = mutableListOf<RangeAttribute>()
        val attributes = base - TextAttribute.FONT
        fun font(family: String?) = buildAwtFont(deckFonts, family ?: paragraph.defaultFontFamily, attributes)
        val logical = Font(attributes + (TextAttribute.FAMILY to Font.SANS_SERIF))
        val remapped = StringBuilder(rawText.length)
        var groupBegin = 0
        for (group in FontGroup.getFontGroupRanges(rawText)) {
            val runFont = run.getFontInfo(group.fontGroup) ?: run.getFontInfo(FontGroup.LATIN)
            val fallbackInfo = fonts.getFallbackFont(graphics, runFont)
            val mappedInfo = fonts.getMappedFont(graphics, runFont)
                ?: fonts.getMappedFont(graphics, PlainFontInfo(paragraph.defaultFontFamily))
                ?: fallbackInfo
            val mapped = font(mappedInfo.typeface)
            val fallback = font(fallbackInfo.typeface)
            val groupEnd = groupBegin + group.length

            fun selectFontAndCodePoint(codePoint: Int): Pair<Font, Int> {
                if (codePoint == '\t'.code) return mapped to codePoint
                if (mapped.canDisplay(codePoint)) return mapped to codePoint
                if (codePoint in SYMBOL_ASCII_MIN..SYMBOL_ASCII_MAX) {
                    val pua = codePoint or SYMBOL_PUA_BASE
                    if (mapped.canDisplay(pua)) return mapped to pua
                } else if (codePoint in SYMBOL_PUA_MIN..SYMBOL_PUA_MAX) {
                    val ascii = codePoint and SYMBOL_ASCII_MAX
                    if (mapped.canDisplay(ascii)) return mapped to ascii
                }
                if (fallback.canDisplay(codePoint)) return fallback to codePoint
                return logical to codePoint
            }

            var partBegin = remapped.length
            val firstPair = selectFontAndCodePoint(rawText.codePointAt(groupBegin))
            var partFont = firstPair.first
            var offset = groupBegin
            while (offset < groupEnd) {
                val codePoint = rawText.codePointAt(offset)
                val (selected, outCodePoint) = selectFontAndCodePoint(codePoint)
                if (selected !== partFont) {
                    if (remapped.length > partBegin) {
                        ranges += RangeAttribute(TextAttribute.FONT, partFont, begin + partBegin, begin + remapped.length)
                    }
                    partBegin = remapped.length
                    partFont = selected
                }
                remapped.appendCodePoint(outCodePoint)
                offset += Character.charCount(codePoint)
            }
            if (remapped.length > partBegin) {
                ranges += RangeAttribute(TextAttribute.FONT, partFont, begin + partBegin, begin + remapped.length)
            }
            groupBegin = groupEnd
        }
        return ResolvedRunText(remapped.toString(), ranges)
    }

    private fun applyTabStops(
        graphics: Graphics2D,
        runText: String,
        begin: Int,
        baseFont: Font,
        startX: Double,
        tabStops: List<Double>,
        defaultTabPt: Double,
        ranges: MutableList<RangeAttribute>,
    ): Double {
        var x = startX
        var segStart = 0
        for (i in runText.indices) {
            val ch = runText[i]
            if (ch == '\n' || ch == '\r') {
                if (i > segStart) {
                    x += TextLayout(runText.substring(segStart, i), baseFont, graphics.fontRenderContext).advance
                }
                x = 0.0
                segStart = i + 1
            } else if (ch == '\t') {
                if (i > segStart) {
                    x += TextLayout(runText.substring(segStart, i), baseFont, graphics.fontRenderContext).advance
                }
                val nextStop = nextTabStop(x, tabStops, defaultTabPt)
                val delta = (nextStop - x).coerceAtLeast(1.0)
                ranges += RangeAttribute(TextAttribute.CHAR_REPLACEMENT, TabGraphicAttribute(delta.toFloat()), begin + i, begin + i + 1)
                x = nextStop
                segStart = i + 1
            }
        }
        if (segStart < runText.length) {
            x += TextLayout(runText.substring(segStart), baseFont, graphics.fontRenderContext).advance
        }
        return x
    }

    private fun renderableTextPreservingTabs(run: TextRun): String {
        val raw = run.rawText.orEmpty().replace('\u000B', '\n')
        return when (run.textCap) {
            TextRun.TextCap.ALL -> raw.uppercase()
            else -> raw
        }
    }
}

@Suppress("UNCHECKED_CAST")
private fun deckEmbeddedFonts(graphics: Graphics2D): Map<String, List<Font>> =
    (graphics.getRenderingHint(DECK_EMBEDDED_FONTS_KEY) as? Map<String, List<Font>>).orEmpty()

private fun buildAwtFont(
    deckFonts: Map<String, List<Font>>,
    family: String,
    attributes: Map<Attribute, Any>,
): Font {
    val cleanKey = family.substringBefore('\u0000').trim().removePrefix("@").lowercase()
    val embedded = deckFonts[cleanKey]
    if (!embedded.isNullOrEmpty()) {
        val wantBold = (attributes[TextAttribute.WEIGHT] as? Float ?: 0f) >= TextAttribute.WEIGHT_BOLD
        val wantItalic = (attributes[TextAttribute.POSTURE] as? Float ?: 0f) > 0f
        val best = embedded.firstOrNull { it.isBold == wantBold && it.isItalic == wantItalic }
            ?: embedded.first()
        val derivedAttrs = attributes - TextAttribute.FONT - TextAttribute.FAMILY
        return best.deriveFont(derivedAttrs)
    }
    return Font((attributes - TextAttribute.FONT) + (TextAttribute.FAMILY to family))
}

internal fun characterSpacingPt(run: TextRun): Double {
    if (run is XSLFTextRun) return run.characterSpacing
    if (run is HSLFTextRun) {
        val style = run.characterStyle ?: return 0.0
        val prop: TextProp = style.findByName<TextProp>("char_spacing")
            ?: style.findByName<TextProp>("text.spacing")
            ?: style.findByName<TextProp>("font.charSpace")
            ?: return 0.0
        // Binary PowerPoint stores character spacing in hundredths of a point.
        return prop.value / 100.0
    }
    return 0.0
}

/** Reads `<a:normAutofit fontScale="..."/>` as a multiplier in (0, 1], defaulting to 1.0. */
internal fun normAutofitFontScale(shape: TextShape<*, *>?): Double {
    val ctShape = (shape as? XSLFTextShape)?.xmlObject as? CTShape ?: return 1.0
    val normAutofit = ctShape.txBody?.bodyPr?.normAutofit ?: return 1.0
    if (!normAutofit.isSetFontScale) return 1.0
    val raw = normAutofit.fontScale.toString().toDoubleOrNull() ?: return 1.0
    val scale = raw / AUTOFIT_PERCENT_BASE
    return if (scale in MIN_AUTOFIT_SCALE..1.0) scale else 1.0
}

/** Reads `<a:normAutofit lnSpcReduction="..."/>` as a fraction in [0, 0.5], defaulting to 0.0. */
internal fun normAutofitLineSpacingReduction(shape: TextShape<*, *>?): Double {
    val ctShape = (shape as? XSLFTextShape)?.xmlObject as? CTShape ?: return 0.0
    val normAutofit = ctShape.txBody?.bodyPr?.normAutofit ?: return 0.0
    if (!normAutofit.isSetLnSpcReduction) return 0.0
    val raw = normAutofit.lnSpcReduction.toString().toDoubleOrNull() ?: return 0.0
    val reduction = raw / AUTOFIT_PERCENT_BASE
    return if (reduction in 0.0..MAX_LINE_SPACING_REDUCTION) reduction else 0.0
}

private fun explicitTabStopsPt(paragraph: TextParagraph<*, *, *>): List<Double> {
    val pPr = (paragraph as? XSLFTextParagraph)?.xmlObject?.pPr ?: return emptyList()
    if (!pPr.isSetTabLst) return emptyList()
    return pPr.tabLst.tabList
        .mapNotNull { tab ->
            if (!tab.isSetPos) null
            else tab.pos.toString().toLongOrNull()?.let { Units.toPoints(it) }
        }
        .sorted()
}

private fun nextTabStop(currentX: Double, explicitStops: List<Double>, defaultTabPt: Double): Double {
    explicitStops.firstOrNull { it > currentX + TAB_EPSILON }?.let { return it }
    val step = defaultTabPt.coerceAtLeast(1.0)
    val index = kotlin.math.floor((currentX + TAB_EPSILON) / step).toLong() + 1L
    return index * step
}

/**
 * The next line, wrapped on its visible width: the spaces that end it may run past [width], by up
 * to [slack] (one em of the paragraph's largest size), as they do in PowerPoint.
 */
private fun LineBreakMeasurer.nextHangingLayout(
    text: String,
    width: Float,
    limit: Int,
    slack: Float,
): TextLayout? {
    val start = position
    var cap = limit
    while (true) {
        val layout = nextLayout(width + slack, cap, true) ?: return null
        if (layout.visibleAdvance <= width) return layout
        val end = position
        position = start
        // A limit inside a word cuts it there, so pull it back to the start of the line's last word.
        val lastWordStart = text.substring(start, end).trimEnd().indexOfLast { it.isWhitespace() } + 1
        if (lastWordStart <= 0) return nextLayout(width, limit, true)
        cap = start + lastWordStart
    }
}

private const val PERCENT = 100.0
private const val AUTOFIT_PERCENT_BASE = 100_000.0
private const val MIN_AUTOFIT_SCALE = 0.2
private const val MAX_LINE_SPACING_REDUCTION = 0.5
private const val DEFAULT_TAB_SIZE_PT = 72.0
private const val TAB_EPSILON = 0.01
private const val SYMBOL_ASCII_MIN = 0x0020
private const val SYMBOL_ASCII_MAX = 0x00FF
private const val SYMBOL_PUA_BASE = 0xF000
private const val SYMBOL_PUA_MIN = 0xF020
private const val SYMBOL_PUA_MAX = 0xF0FF

private val LINE_BREAKS = Regex("[\\r\\n]")

/** A font known only by its typeface — POI's `DrawFontInfo`, which is internal to its package. */
private class PlainFontInfo(private val name: String) : FontInfo {
    override fun getTypeface(): String = name
}
