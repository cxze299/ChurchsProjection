package org.churchpresenter.bibletab

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.VerticalScrollbar
import androidx.compose.foundation.rememberScrollbarAdapter
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.Alignment
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt
import org.churchpresenter.core.models.bible.SelectedVerse
import org.churchpresenter.presenter.BiblePresenter
import org.churchpresenter.settings.AppSettings
import org.churchpresenter.settings.BibleTranslationSettings
import org.churchpresenter.sharedui.composables.ColorPickerField
import org.churchpresenter.sharedui.composables.DropdownSettingsField
import org.churchpresenter.sharedui.composables.FontSettingsDropdown
import org.churchpresenter.sharedui.composables.NumberSettingsTextField
import org.churchpresenter.sharedui.composables.SlimSlider
import org.churchpresenter.sharedui.composables.TextStyleButtons
import org.churchpresenter.sharedui.utils.rememberSystemFonts
import org.churchpresenter.strings.generated.resources.Res
import org.churchpresenter.strings.generated.resources.bible_projection_style_title
import org.churchpresenter.strings.generated.resources.bible_projection_style_no_translation
import org.churchpresenter.strings.generated.resources.bible_projection_style_translation
import org.churchpresenter.strings.generated.resources.bible_projection_style_text_size
import org.churchpresenter.strings.generated.resources.bible_projection_style_reference_size
import org.churchpresenter.strings.generated.resources.bible_projection_style_line_spacing
import org.churchpresenter.strings.generated.resources.bible_projection_style_passage_spacing
import org.churchpresenter.strings.generated.resources.apply
import org.churchpresenter.strings.generated.resources.cancel
import org.churchpresenter.strings.generated.resources.preview
import org.churchpresenter.strings.generated.resources.bible_projection_style_sample
import org.churchpresenter.strings.generated.resources.bible_projection_style_sample_book
import org.churchpresenter.strings.generated.resources.bible_projection_style_sample_next
import org.churchpresenter.strings.generated.resources.font_type
import org.churchpresenter.strings.generated.resources.text_color
import org.jetbrains.compose.resources.stringResource

internal fun updateBibleProjectionTranslation(
    settings: AppSettings,
    fileName: String,
    styleKey: String,
    transform: (BibleTranslationSettings) -> BibleTranslationSettings,
): AppSettings {
    val index = settings.bibleSettings.translationList().indexOfFirst { it.fileName == fileName }
    return settings.copy(bibleSettings = settings.bibleSettings.updateTranslation(index) {
        transform(it).copy(ownStyleKeys = it.ownStyleKeys + styleKey)
    })
}

@Composable
internal fun BibleProjectionStyleDialog(
    settings: AppSettings,
    onSettingsChange: ((AppSettings) -> AppSettings) -> Unit,
    onDismiss: () -> Unit,
) {
    val changes = remember { mutableStateMapOf<Pair<String, String>, (AppSettings) -> AppSettings>() }
    val draft = changes.values.fold(settings) { current, change -> change(current) }
    val bible = draft.bibleSettings
    val stack = bible.translationList()
    var selectedFile by remember { mutableStateOf(stack.firstOrNull()?.fileName.orEmpty()) }
    val index = stack.indexOfFirst { it.fileName == selectedFile }.coerceAtLeast(0)
    val translation = stack.getOrNull(index)
    val fonts = rememberSystemFonts()
    fun edit(key: String, transform: (BibleTranslationSettings) -> BibleTranslationSettings) {
        val file = translation?.fileName ?: return
        changes[file to key] = { current -> updateBibleProjectionTranslation(current, file, key, transform) }
    }
    AlertDialog(
        modifier = Modifier.testTag("bible_projection_style_dialog"),
        onDismissRequest = onDismiss,
        title = { Text(stringResource(Res.string.bible_projection_style_title)) },
        confirmButton = {
            TextButton(enabled = changes.isNotEmpty(), onClick = {
                val edits = changes.values.toList()
                onSettingsChange { current -> edits.fold(current) { value, edit -> edit(value) } }
                onDismiss()
            }) { Text(stringResource(Res.string.apply)) }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text(stringResource(Res.string.cancel)) } },
        text = {
            Column(Modifier.fillMaxWidth().heightIn(max = 520.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)) {
                if (translation == null) {
                    Text(stringResource(Res.string.bible_projection_style_no_translation))
                } else {
                    Text(stringResource(Res.string.preview))
                    val sample = stringResource(Res.string.bible_projection_style_sample)
                    val book = stringResource(Res.string.bible_projection_style_sample_book)
                    val first = SelectedVerse(translationFileName = translation.fileName, bookName = book,
                        chapter = 119, verseNumber = 105, verseText = sample)
                    val second = first.copy(verseNumber = 106,
                        verseText = stringResource(Res.string.bible_projection_style_sample_next))
                    BiblePresenter(
                        modifier = Modifier.fillMaxWidth().aspectRatio(2.4f).testTag("bible_projection_preview"),
                        selectedVerses = listOf(first.copy(continuousPassages = listOf(listOf(first), listOf(second)))),
                        appSettings = draft.copy(bibleSettings = bible.withTranslations(listOf(translation))),
                    )
                    val scroll = rememberScrollState()
                    Box(Modifier.weight(1f, fill = false)) {
                        Column(Modifier.fillMaxWidth().verticalScroll(scroll).padding(end = 14.dp),
                            verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            val labels = stack.mapIndexed { position, item ->
                                "${position + 1} · ${item.customName.ifBlank { item.fileName.substringBeforeLast('.') }}"
                            }
                            if (stack.size > 1) {
                                DropdownSettingsField(value = labels[index], options = labels,
                                    label = stringResource(Res.string.bible_projection_style_translation),
                                    onValueChange = { label -> selectedFile = stack[labels.indexOf(label)].fileName })
                            }
                            FontSettingsDropdown(label = stringResource(Res.string.font_type), value = translation.textFontType,
                                fonts = fonts, modifier = Modifier.fillMaxWidth(), fillWidth = true,
                                onValueChange = { value -> edit("textFontType") { it.copy(textFontType = value) } })
                            StyleSlider(label = stringResource(Res.string.bible_projection_style_text_size),
                                tag = "bible_projection_text_size",
                                initialText = translation.textFontSize, range = 8..240,
                                onValueChange = { value -> edit("textFontSize") { it.copy(textFontSize = value) } })
                            StyleSlider(label = stringResource(Res.string.bible_projection_style_reference_size),
                                tag = "bible_projection_reference_size",
                                initialText = translation.referenceFontSize, range = 8..240,
                                onValueChange = { value -> edit("referenceFontSize") { it.copy(referenceFontSize = value) } })
                            StyleSlider(label = stringResource(Res.string.bible_projection_style_line_spacing),
                                tag = "bible_projection_line_spacing",
                                initialText = bible.continuousLineSpacingPercent ?: 0, range = 0..300,
                                onValueChange = { value -> if (value == 0 || value >= 50) changes["" to "lineSpacing"] = { current -> current.copy(
                                    bibleSettings = current.bibleSettings.copy(continuousLineSpacingPercent = value.takeIf { it > 0 })) } })
                            StyleSlider(label = stringResource(Res.string.bible_projection_style_passage_spacing),
                                tag = "bible_projection_passage_spacing",
                                initialText = bible.continuousPassageSpacing ?: bible.multiTranslationSpacing, range = 0..200,
                                onValueChange = { value -> changes["" to "passageSpacing"] = { current -> current.copy(
                                    bibleSettings = current.bibleSettings.copy(continuousPassageSpacing = value)) } })
                            ColorPickerField(color = translation.textColor, label = stringResource(Res.string.text_color),
                                onColorChange = { value -> edit("textColor") { it.copy(textColor = value) } })
                            TextStyleButtons(bold = translation.textBold, italic = translation.textItalic,
                                underline = translation.textUnderline, shadow = translation.textShadow,
                                onBoldChange = { value -> edit("textBold") { it.copy(textBold = value) } },
                                onItalicChange = { value -> edit("textItalic") { it.copy(textItalic = value) } },
                                onUnderlineChange = { value -> edit("textUnderline") { it.copy(textUnderline = value) } },
                                onShadowChange = { value -> edit("textShadow") { it.copy(textShadow = value) } })
                        }
                        if (scroll.maxValue in 1 until Int.MAX_VALUE) {
                            Box(Modifier.matchParentSize(), contentAlignment = Alignment.CenterEnd) {
                                VerticalScrollbar(rememberScrollbarAdapter(scroll), Modifier.fillMaxHeight())
                            }
                        }
                    }
                }
            }
        },
    )
}

@Composable
private fun StyleSlider(label: String, tag: String, initialText: Int, range: IntRange, onValueChange: (Int) -> Unit) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        NumberSettingsTextField(
            modifier = Modifier.widthIn(min = 140.dp).width(IntrinsicSize.Max).testTag(tag),
            label = label, initialText = initialText, range = range, onValueChange = onValueChange,
        )
        SlimSlider(
            value = initialText.toFloat(),
            onValueChange = { value ->
                val number = value.roundToInt()
                onValueChange(if (tag == "bible_projection_line_spacing" && number in 1..49) {
                    if (number < 25) 0 else 50
                } else number)
            },
            valueRange = range.first.toFloat()..range.last.toFloat(),
            modifier = Modifier.weight(1f).testTag("${tag}_slider").semantics { contentDescription = label },
        )
    }
}
