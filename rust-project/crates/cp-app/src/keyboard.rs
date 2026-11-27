use crate::app::{App, Tab};
use cp_core::{KeyChord, KeyboardShortcutSettings, ShortcutAction};
use egui::{Context, Event, Key};

pub fn chords(settings: &KeyboardShortcutSettings, action: ShortcutAction) -> Vec<KeyChord> {
    let mut list = settings.chords_for(action);
    if action == ShortcutAction::NextItem && !settings.overrides.contains_key(action.key_name()) {
        list.push(KeyChord::new(32, false, false, false, false));
    }
    list
}
fn code(key: Key) -> Option<i64> {
    use Key::*;
    Some(match key {
        Escape => 27,
        Space => 32,
        Enter => 13,
        Tab => 9,
        Backspace => 8,
        Delete => 46,
        Insert => 45,
        Home => 36,
        End => 35,
        ArrowLeft => 37,
        ArrowUp => 38,
        ArrowRight => 39,
        ArrowDown => 40,
        PageUp => 33,
        PageDown => 34,
        Comma => 188,
        Period => 190,
        Minus => 189,
        Slash | Questionmark => 191,
        Backslash | Pipe => 220,
        OpenBracket | OpenCurlyBracket => 219,
        CloseBracket | CloseCurlyBracket => 221,
        Quote => 222,
        Semicolon | Colon => 186,
        Equals | Plus => 187,
        Backtick => 192,
        Exclamationmark => 49,
        _ if (Key::A..=Key::Z).contains(&key) => 65 + key as i64 - Key::A as i64,
        _ if (Key::Num0..=Key::Num9).contains(&key) => 48 + key as i64 - Key::Num0 as i64,
        _ if (Key::F1..=Key::F35).contains(&key) => 112 + key as i64 - Key::F1 as i64,
        _ => return None,
    })
}
// Compose Desktop stores the AWT virtual key in the low word and the standard
// key location in the high word. cp-core's built-in defaults use Windows VK codes.
fn compose_code(key: Key) -> Option<i64> {
    use Key::*;
    let awt = match key {
        Enter => 10,
        Delete => 127,
        Insert => 155,
        Minus => 45,
        Comma => 44,
        Period => 46,
        Slash | Questionmark => 47,
        Backslash | Pipe => 92,
        OpenBracket | OpenCurlyBracket => 91,
        CloseBracket | CloseCurlyBracket => 93,
        Quote => 222,
        Semicolon => 59,
        Colon => 513,
        Equals => 61,
        Plus => 521,
        Backtick => 192,
        Exclamationmark => 517,
        _ if (F13..=F24).contains(&key) => 61440 + key as i64 - F13 as i64,
        _ if (F25..=F35).contains(&key) => return None,
        _ => code(key)?,
    };
    Some((1_i64 << 32) | awt)
}

pub fn label(chord: &KeyChord) -> String {
    let name = Key::ALL
        .iter()
        .find(|&&k| compose_code(k) == Some(chord.key_code) || code(k) == Some(chord.key_code));
    if let Some(key) = name {
        let mut parts = Vec::new();
        if chord.ctrl {
            parts.push("Ctrl");
        }
        if chord.alt {
            parts.push("Alt");
        }
        if chord.shift {
            parts.push("Shift");
        }
        if chord.meta {
            parts.push("Meta");
        }
        parts.push(key.name());
        parts.join("+")
    } else {
        chord.format_label()
    }
}
impl App {
    pub fn keyboard(&mut self, ctx: &Context, projector: bool) {
        let events = ctx.input(|i| i.events.clone());
        let editing = ctx
            .memory(|m| m.focused())
            .is_some_and(|id| egui::TextEdit::load_state(ctx, id).is_some());
        for event in events {
            let Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } = event
            else {
                continue;
            };
            let Some(key_code) = compose_code(key) else {
                continue;
            };
            let key_chord = KeyChord::new(
                key_code,
                modifiers.ctrl,
                modifiers.shift,
                modifiers.alt,
                modifiers.mac_cmd,
            );
            let legacy = KeyChord {
                key_code: code(key).unwrap_or(0),
                ..key_chord.clone()
            };
            let matches = |c: &KeyChord| c == &key_chord || c == &legacy;
            let prefs = &self.settings.keyboard_shortcut_settings;
            if !projector {
                if let Some(action) = self.record.take() {
                    let conflict = ShortcutAction::ALL
                        .iter()
                        .copied()
                        .find(|&a| a != action && chords(prefs, a).iter().any(matches));
                    if let Some(other) = conflict {
                        self.message =
                            format!("组合键已用于「{}」，请重新录制", other.display_name());
                        self.record = Some(action);
                    } else {
                        self.settings
                            .keyboard_shortcut_settings
                            .set_chord(action, key_chord);
                        self.message.clear();
                        self.save();
                    }
                    ctx.input_mut(|i| {
                        i.consume_key(modifiers, key);
                    });
                    return;
                }
                if self.style_open {
                    continue;
                }
            }
            let Some(action) = ShortcutAction::ALL
                .iter()
                .copied()
                .find(|a| chords(prefs, *a).iter().any(matches))
            else {
                continue;
            };
            if !projector
                && editing
                && !modifiers.ctrl
                && !modifiers.mac_cmd
                && !modifiers.alt
                && !matches!(
                    action,
                    ShortcutAction::HideProjection | ShortcutAction::ToggleFullscreen
                )
            {
                continue;
            }
            ctx.input_mut(|i| {
                i.consume_key(modifiers, key);
            });
            match action {
                ShortcutAction::GoLive => match self.tab {
                    Tab::Bible => {
                        if editing && !self.quick.is_empty() {
                            self.submit_bible(true);
                        } else {
                            self.go_bible();
                        }
                    }
                    Tab::Library | Tab::Songs => self.go_media(),
                    Tab::Pictures => self.go_picture(),
                    Tab::Announcements => self.go_announcement(),
                    Tab::Settings => {
                        self.output.visible = true;
                        self.output.blank = false;
                    }
                },
                ShortcutAction::HideProjection => {
                    self.output.visible = false;
                    self.cancel_pending_live();
                }
                ShortcutAction::BlackScreen => self.output.blank = !self.output.blank,
                ShortcutAction::ToggleFullscreen => {
                    self.fullscreen = !self.fullscreen;
                    self.target_changed();
                }
                ShortcutAction::ToggleAlwaysOnTop => self.output.top = !self.output.top,
                ShortcutAction::ClearPassages => self.clear_passages(),
                ShortcutAction::FocusSearch => {
                    if matches!(self.tab, Tab::Settings | Tab::Pictures | Tab::Announcements) {
                        self.change_tab(Tab::Bible);
                    }
                    self.focus_search = true;
                }
                ShortcutAction::NextItem => self.navigate(1, projector),
                ShortcutAction::PreviousItem => self.navigate(-1, projector),
            }
            ctx.request_repaint_of(egui::ViewportId::ROOT);
        }
    }
}
