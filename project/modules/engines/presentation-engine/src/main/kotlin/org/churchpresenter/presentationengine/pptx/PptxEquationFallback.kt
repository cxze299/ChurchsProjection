package org.churchpresenter.presentationengine.pptx

import org.apache.poi.xslf.usermodel.XMLSlideShow
import org.apache.xmlbeans.XmlObject
import javax.xml.namespace.QName

private const val MC = "http://schemas.openxmlformats.org/markup-compatibility/2006"
private const val P = "http://schemas.openxmlformats.org/presentationml/2006/main"
private const val MATH = "http://schemas.openxmlformats.org/officeDocument/2006/math"
private val alternateContent = "declare namespace mc='$MC' .//mc:AlternateContent"
private val officeMath = "declare namespace m='$MATH' .//m:oMath"
private val choice = QName(MC, "Choice")
private val alternateContentName = QName(MC, "AlternateContent")
private val fallback = QName(MC, "Fallback")
private val shape = QName(P, "sp")
private val picture = QName(P, "pic")
private val groupShape = QName(P, "grpSp")

/** Promote the authored fallback for an Office Math shape that POI cannot draw. */
internal fun preferEquationFallbacks(xml: XmlObject) {
    for (alternate in xml.selectPath(alternateContent)) {
        if (alternate.selectChildren(choice).none { it.selectPath(officeMath).isNotEmpty() }) continue
        val replacement = alternate.selectChildren(fallback).singleOrNull() ?: continue
        alternate.newCursor().use { destination ->
            if (!destination.isStart) destination.toFirstChild()
            if (destination.name != alternateContentName) return@use
            replacement.newCursor().use sourceCursor@{ source ->
                if (!source.toFirstChild() ||
                    (source.name != shape && source.name != picture && source.name != groupShape)
                ) return@sourceCursor
                source.push()
                val hasOtherChildren = source.toNextSibling()
                source.pop()
                if (hasOtherChildren) return@sourceCursor
                if (source.copyXml(destination)) destination.removeXml()
            }
        }
    }
}

internal fun preferEquationFallbacks(show: XMLSlideShow) {
    val seen = mutableSetOf<XmlObject>()
    for (slide in show.slides) {
        runCatching {
            val layout = slide.slideLayout
            for (xml in listOf(slide.xmlObject, layout.xmlObject, layout.slideMaster.xmlObject)) {
                if (seen.add(xml)) preferEquationFallbacks(xml)
            }
        }
    }
}
