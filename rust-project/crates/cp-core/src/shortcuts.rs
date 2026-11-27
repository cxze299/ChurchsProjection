use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KeyChord {
    #[serde(default)]
    pub key_code: i64,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub meta: bool,
}

impl KeyChord {
    pub fn new(key_code: i64, ctrl: bool, shift: bool, alt: bool, meta: bool) -> Self {
        Self {
            key_code,
            ctrl,
            shift,
            alt,
            meta,
        }
    }

    pub fn format_label(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.meta {
            parts.push("Win".to_string());
        }
        parts.push(key_code_name(self.key_code));
        parts.join("+")
    }
}

pub fn key_code_name(code: i64) -> String {
    match code {
        27 => "Esc".to_string(),
        32 => "Space".to_string(),
        13 => "Enter".to_string(),
        33 => "PageUp".to_string(),
        34 => "PageDown".to_string(),
        37 => "Left".to_string(),
        38 => "Up".to_string(),
        39 => "Right".to_string(),
        40 => "Down".to_string(),
        65..=90 => char::from_u32(code as u32)
            .map(|c| c.to_string())
            .unwrap_or_else(|| format!("Key({})", code)),
        112..=123 => format!("F{}", code - 111),
        _ => format!("Key({})", code),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShortcutAction {
    GoLive,
    HideProjection,
    BlackScreen,
    ToggleFullscreen,
    ToggleAlwaysOnTop,
    NextItem,
    PreviousItem,
    ClearPassages,
    FocusSearch,
}

impl ShortcutAction {
    pub const ALL: &'static [ShortcutAction] = &[
        ShortcutAction::GoLive,
        ShortcutAction::HideProjection,
        ShortcutAction::BlackScreen,
        ShortcutAction::ToggleFullscreen,
        ShortcutAction::ToggleAlwaysOnTop,
        ShortcutAction::NextItem,
        ShortcutAction::PreviousItem,
        ShortcutAction::ClearPassages,
        ShortcutAction::FocusSearch,
    ];

    pub fn key_name(&self) -> &'static str {
        match self {
            ShortcutAction::GoLive => "GO_LIVE",
            ShortcutAction::HideProjection => "HIDE_PROJECTION",
            ShortcutAction::BlackScreen => "BLACK_SCREEN",
            ShortcutAction::ToggleFullscreen => "TOGGLE_FULLSCREEN",
            ShortcutAction::ToggleAlwaysOnTop => "TOGGLE_ALWAYS_ON_TOP",
            ShortcutAction::NextItem => "NEXT_ITEM",
            ShortcutAction::PreviousItem => "PREVIOUS_ITEM",
            ShortcutAction::ClearPassages => "CLEAR_PASSAGES",
            ShortcutAction::FocusSearch => "FOCUS_SEARCH",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ShortcutAction::GoLive => "立即上屏 / 恢复投影",
            ShortcutAction::HideProjection => "隐藏投影窗口 (Esc)",
            ShortcutAction::BlackScreen => "临时黑屏 / 清除文字 (B)",
            ShortcutAction::ToggleFullscreen => "切换全屏 / 窗口模式 (F11)",
            ShortcutAction::ToggleAlwaysOnTop => "切换投影窗口置顶 (T)",
            ShortcutAction::NextItem => "下一节 / 下一页 (↓ / PageDown / Space)",
            ShortcutAction::PreviousItem => "上一节 / 上一页 (↑ / PageUp)",
            ShortcutAction::ClearPassages => "清除累计连续经文",
            ShortcutAction::FocusSearch => "聚焦快速检索框 (Ctrl+F)",
        }
    }

    pub fn default_chords(&self) -> Vec<KeyChord> {
        match self {
            ShortcutAction::GoLive => vec![KeyChord::new(13, true, false, false, false)], // Ctrl+Enter
            ShortcutAction::HideProjection => vec![KeyChord::new(27, false, false, false, false)], // Esc
            ShortcutAction::BlackScreen => vec![KeyChord::new(66, false, false, false, false)], // B
            ShortcutAction::ToggleFullscreen => vec![KeyChord::new(122, false, false, false, false)], // F11
            ShortcutAction::ToggleAlwaysOnTop => vec![KeyChord::new(84, false, false, false, false)], // T
            ShortcutAction::NextItem => vec![
                KeyChord::new(40, false, false, false, false), // Down
                KeyChord::new(34, false, false, false, false), // PageDown
            ],
            ShortcutAction::PreviousItem => vec![
                KeyChord::new(38, false, false, false, false), // Up
                KeyChord::new(33, false, false, false, false), // PageUp
            ],
            ShortcutAction::ClearPassages => vec![KeyChord::new(76, true, false, false, false)], // Ctrl+L
            ShortcutAction::FocusSearch => vec![KeyChord::new(70, true, false, false, false)], // Ctrl+F
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardShortcutSettings {
    #[serde(default)]
    pub overrides: HashMap<String, Vec<KeyChord>>,
    #[serde(default = "default_true")]
    pub focus_search_on_tab_open: bool,
}

fn default_true() -> bool {
    true
}

impl Default for KeyboardShortcutSettings {
    fn default() -> Self {
        Self {
            overrides: HashMap::new(),
            focus_search_on_tab_open: true,
        }
    }
}

impl KeyboardShortcutSettings {
    pub fn chords_for(&self, action: ShortcutAction) -> Vec<KeyChord> {
        if let Some(custom) = self.overrides.get(action.key_name()) {
            custom.clone()
        } else {
            action.default_chords()
        }
    }

    pub fn set_chord(&mut self, action: ShortcutAction, chord: KeyChord) {
        self.overrides
            .insert(action.key_name().to_string(), vec![chord]);
    }

    pub fn clear_action(&mut self, action: ShortcutAction) {
        self.overrides
            .insert(action.key_name().to_string(), Vec::new());
    }

    pub fn reset_action(&mut self, action: ShortcutAction) {
        self.overrides.remove(action.key_name());
    }
}
