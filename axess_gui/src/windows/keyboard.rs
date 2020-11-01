use std::{borrow::Cow, fmt::Display};

use axess_core::payload::{DeviceState, UiPayload};

use packed_struct::PrimitiveEnum;

use crate::config::AxessConfiguration;

#[derive(Debug, Copy, Clone)]
pub enum UiEvent {
    KeyDown(usize, u32),
    KeyUp(usize, u32)
}

#[derive(Default, Debug)]
pub struct KeyboardState {
    ctrl: bool
}

#[derive(Debug, Copy, Clone)]
pub enum KeyboardCombination {
    Key(usize),
    CtrlKey(usize)
}

impl KeyboardState {
    pub fn handle_event(&mut self, ev: &UiEvent) -> Option<KeyboardCombination> {
        const CTRL: usize = 0x11;
        const ALT: usize = 0x12;

        match *ev {
            UiEvent::KeyDown(CTRL, _) => {
                self.ctrl = true;
            }
            UiEvent::KeyUp(CTRL, _) => {
                self.ctrl = false;
            }
            UiEvent::KeyDown(k, _) =>  {
                // todo: ignore repeats?

                if self.ctrl {
                    return Some(KeyboardCombination::CtrlKey(k));
                } else {
                    return Some(KeyboardCombination::Key(k));
                }
            },
            UiEvent::KeyUp(_, _) => {}
        }

        None
    }
}


#[derive(Debug, Copy, Clone, PartialEq, PrimitiveEnum)]
pub enum Keys {
    Enter = 13,
    PageUp = 33,
    PageDown = 34,
    Space = 32,
    Tab = 9,
    Number0 = 48,
    Number1 = 49,
    Number2 = 50,
    Number3 = 51,
    Number4 = 52,
    Number5 = 53,
    Number6 = 54,
    Number7 = 55,
    Number8 = 56,   
    Number9 = 57,
    Fn1 = 0x70,
    Fn2 = 0x71,
    Fn3 = 0x72,
    Fn4 = 0x73,
    Fn5 = 0x74,
    Fn6 = 0x75,
    Fn7 = 0x76,
    Fn8 = 0x77,
    Fn9 = 0x78,
    Fn10 = 0x79,
    Fn11 = 0x7A,
    Fn12 = 0x7B,
    Fn13 = 0x7C,
    Fn14 = 0x7D,
    Fn15 = 0x7E,
    Fn16 = 0x7F
}

impl Display for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Keys::Number0 => "0",
            Keys::Number1 => "1",
            Keys::Number2 => "2",
            Keys::Number3 => "3",
            Keys::Number4 => "4",
            Keys::Number5 => "5",
            Keys::Number6 => "6",
            Keys::Number7 => "7",
            Keys::Number8 => "8",
            Keys::Number9 => "9",
            Keys::PageUp => "Page Up",
            Keys::PageDown => "Page Down",
            Keys::Fn1 => "F1",
            Keys::Fn2 => "F2",
            Keys::Fn3 => "F3",
            Keys::Fn4 => "F4",
            Keys::Fn5 => "F5",
            Keys::Fn6 => "F6",
            Keys::Fn7 => "F7",
            Keys::Fn8 => "F8",
            Keys::Fn9 => "F9",
            Keys::Fn10 => "F10",
            Keys::Fn11 => "F11",
            Keys::Fn12 => "F12",
            Keys::Fn13 => "F13",
            Keys::Fn14 => "F14",
            Keys::Fn15 => "F15",
            Keys::Fn16 => "F16",
            _ => { return f.write_fmt(format_args!("{:?}", self)); }
        };
        f.write_str(str)
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    pub key: KeyboardShortcutKey,
    pub command_description: Cow<'static, str>,
    pub command: ShortcutCommand
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyboardShortcutKey {
    Key(Keys),
    CtrlKey(Keys)
}

impl Display for KeyboardShortcutKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyboardShortcutKey::Key(k) => {
                k.fmt(f)
            }
            KeyboardShortcutKey::CtrlKey(k) => {
                f.write_str("Ctrl + ")?;
                k.fmt(f)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ShortcutCommand {
    UiPayload(UiPayload),
    SelectPresetOrScene
}

pub fn get_main_keyboard_shortcuts(config: &AxessConfiguration) -> Vec<KeyboardShortcut> {
    let mut s = vec![
        KeyboardShortcut {
            key: KeyboardShortcutKey::Key(Keys::Enter),
            command_description: "Select the preset or scene".into(),
            command: ShortcutCommand::SelectPresetOrScene
        }
    ];

    if config.keyboard_shortcuts_axe_edit {

        s.push(KeyboardShortcut {
            key: KeyboardShortcutKey::CtrlKey(Keys::PageUp),
            command_description: "Preset Up".into(),
            command: ShortcutCommand::UiPayload(UiPayload::DeviceState(DeviceState::DeltaPreset { delta: 1 }))
        });

        s.push(KeyboardShortcut {
            key: KeyboardShortcutKey::CtrlKey(Keys::PageDown),
            command_description: "Preset Down".into(),
            command: ShortcutCommand::UiPayload(UiPayload::DeviceState(DeviceState::DeltaPreset { delta: -1 }))
        });

        for i in 0..8 {
            let key = Keys::from_primitive(Keys::Number1.to_primitive() + i);
            if let Some(key) = key {
                s.push(KeyboardShortcut {
                    key: KeyboardShortcutKey::CtrlKey(key),
                    command_description: format!("Select Scene {}", i + 1).into(),
                    command: ShortcutCommand::UiPayload(UiPayload::DeviceState(DeviceState::SetScene { scene: i }))
                });
            }
        }

    }

    if config.keyboard_shortcuts_presets_and_scenes_function_keys {
        for i in 0..8 {
            let key = Keys::from_primitive(Keys::Fn1.to_primitive() + i);
            if let Some(key) = key {
                s.push(KeyboardShortcut {
                    key: KeyboardShortcutKey::Key(key),
                    command_description: format!("Select Scene {}", i + 1).into(),
                    command: ShortcutCommand::UiPayload(UiPayload::DeviceState(DeviceState::SetScene { scene: i }))
                });
            }
        }

        s.push(KeyboardShortcut {
            key: KeyboardShortcutKey::Key(Keys::Fn9),
            command_description: "Preset Down".into(),
            command: ShortcutCommand::UiPayload(UiPayload::DeviceState(DeviceState::DeltaPreset { delta: -1 }))
        });

        s.push(KeyboardShortcut {
            key: KeyboardShortcutKey::Key(Keys::Fn10),
            command_description: "Preset Up".into(),
            command: ShortcutCommand::UiPayload(UiPayload::DeviceState(DeviceState::DeltaPreset { delta: 1 }))
        });
    }
    
    s
}