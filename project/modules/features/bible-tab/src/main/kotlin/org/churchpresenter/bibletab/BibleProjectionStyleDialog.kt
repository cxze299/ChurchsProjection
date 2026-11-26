package org.churchpresenter.bibletab

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.ExperimentalLayoutApi
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
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import org.churchpresenter.settings.AppSettings
import org.churchpresenter.settings.BibleTranslationSettings
import org.churchpresenter.sharedui.composables.ColorPickerField
import org.churchpresenter.sharedui.composables.DropdownSettingsField
import org.churchpresenter.sharedui.composables.FontSettingsDropdown
import org.churchpresenter.sharedui.composables.NumberSettingsTextField
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
import org.churchpresenter.strings.generated.resources.close
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
@OptIn(ExperimentalLayoutApi::class)
internal fun BibleProjectionStyleDialog(
    settings: AppSettings,
    onSettingsChange: ((AppSettings) -> AppSettings) -> Unit,
    onDismiss: () -> Unit,
) {
    val bible = settings.bibleSettings
    val stack = bible.translationList()
    var selectedFile by remember { mutableStateOf(stack.firstOrNull()?.fileName.orEmpty()) }
    val index = stack.indexOfFirst { it.fileName == selectedFile }.coerceAtLeast(0)
    val translation = stack.getOrNull(index)
    val fonts = rememberSystemFonts()
    fun edit(key: String, transform: (BibleTranslationSettings) -> BibleTranslationSettings) {
        val file = translation?.fileName ?: return
        onSettingsChange { current -> updateBibleProjectionTranslation(current, file, key, transform) }
    }
    AlertDialog(
        modifier = Modifier.testTag("bible_projection_style_dialog"),
        onDismissRequest = onDismiss,
        title = { Text(stringResource(Res.string.bible_projection_style_title)) },
        confirmButton = { TextButton(onClick = onDismiss) { Text(stringResource(Res.string.close)) } },
        text = {
            Column(Modifier.fillMaxWidth().heightIn(max = 520.dp).verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(12.dp)) {
                if (translation == null) {
                    Text(stringResource(Res.string.bible_projection_style_no_translation))
                } else {
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
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        NumberSettingsTextField(label = stringResource(Res.string.bible_projection_style_text_size),
                            modifier = styleNumberModifier("bible_projection_text_size"),
                            initialText = translation.textFontSize, range = 8..240,
                            onValueChange = { value -> edit("textFontSize") { it.copy(textFontSize = value) } })
                        NumberSettingsTextField(label = stringResource(Res.string.bible_projection_style_reference_size),
                            modifier = styleNumberModifier("bible_projection_reference_size"),
                            initialText = translation.referenceFontSize, range = 8..240,
                            onValueChange = { value -> edit("referenceFontSize") { it.copy(referenceFontSize = value) } })
                        NumberSettingsTextField(label = stringResource(Res.string.bible_projection_style_line_spacing),
                            modifier = styleNumberModifier("bible_projection_line_spacing"),
                            initialText = bible.continuousLineSpacingPercent ?: 0, range = 0..300,
                            onValueChange = { value -> if (value == 0 || value >= 50) onSettingsChange { current -> current.copy(
                                bibleSettings = current.bibleSettings.copy(continuousLineSpacingPercent = value.takeIf { it > 0 })) } })
                        NumberSettingsTextField(label = stringResource(Res.string.bible_projection_style_passage_spacing),
                            modifier = styleNumberModifier("bible_projection_passage_spacing"),
                            initialText = bible.continuousPassageSpacing ?: bible.multiTranslationSpacing, range = 0..200,
                            onValueChange = { value -> onSettingsChange { current -> current.copy(
                                bibleSettings = current.bibleSettings.copy(continuousPassageSpacing = value)) } })
                    }
                    ColorPickerField(color = translation.textColor, label = stringResource(Res.string.text_color),
                        onColorChange = { value -> edit("textColor") { it.copy(textColor = value) } })
                    TextStyleButtons(bold = translation.textBold, italic = translation.textItalic,
                        underline = translation.textUnderline, shadow = translation.textShadow,
                        onBoldChange = { value -> edit("textBold") { it.copy(textBold = value) } },
                        onItalicChange = { value -> edit("textItalic") { it.copy(textItalic = value) } },
                        onUnderlineChange = { value -> edit("textUnderline") { it.copy(textUnderline = value) } },
                        onShadowChange = { value -> edit("textShadow") { it.copy(textShadow = value) } })
                }
            }
        },
    )
}

private fun styleNumberModifier(tag: String): Modifier =
    Modifier.widthIn(min = 120.dp).width(IntrinsicSize.Max).testTag(tag)
