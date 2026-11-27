package org.churchpresenter.presentationengine.pptx

import org.apache.xmlbeans.XmlObject
import javax.xml.namespace.QName
import kotlin.test.Test
import kotlin.test.assertEquals

class PptxEquationFallbackTest {
    @Test
    fun `equation artwork replaces the unsupported shape in place and preserves its relationship`() {
        val xml = XmlObject.Factory.parse(
            """
            <p:spTree xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                      xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"
                      xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
                      xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
                <p:sp id="before"/>
                <mc:AlternateContent>
                    <mc:Choice Requires="m"><p:sp><m:oMath/></p:sp></mc:Choice>
                    <mc:Fallback><p:sp r:embed="rId6"/></mc:Fallback>
                </mc:AlternateContent>
                <mc:AlternateContent>
                    <mc:Choice Requires="p"><p:pic/></mc:Choice>
                    <mc:Fallback><p:sp/></mc:Fallback>
                </mc:AlternateContent>
                <p:sp id="after"/>
            </p:spTree>
            """.trimIndent()
        )
        preferEquationFallbacks(xml)

        xml.newCursor().use { cursor ->
            cursor.toFirstChild()
            cursor.toFirstChild()
            assertEquals("before", cursor.getAttributeText(QName("id")))
            cursor.toNextSibling()
            assertEquals("sp", cursor.name.localPart)
            assertEquals("rId6", cursor.getAttributeText(QName(
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships", "embed",
            )))
            cursor.toNextSibling()
            assertEquals("AlternateContent", cursor.name.localPart)
            cursor.toNextSibling()
            assertEquals("after", cursor.getAttributeText(QName("id")))
        }
    }
}
