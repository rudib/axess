use std::borrow::Cow;

use axess_core::payload::{DeviceState, UiPayload};

use packed_struct::PrimitiveEnum;

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
    Number9 = 57
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

#[derive(Debug, Clone)]
pub enum ShortcutCommand {
    UiPayload(UiPayload),
    SelectPresetOrScene
}

pub fn get_main_keyboard_shortcuts() -> Vec<KeyboardShortcut> {
    let mut s = vec![
        KeyboardShortcut {
            key: KeyboardShortcutKey::Key(Keys::Enter),
            command_description: "Select the preset or scene".into(),
            command: ShortcutCommand::SelectPresetOrScene
        },
        KeyboardShortcut {
            key: KeyboardShortcutKey::CtrlKey(Keys::PageUp),
            command_description: "Preset Up".into(),
            command: ShortcutCommand::UiPayload(UiPayload::DeviceState(DeviceState::DeltaPreset { delta: 1 }))
        },
        KeyboardShortcut {
            key: KeyboardShortcutKey::CtrlKey(Keys::PageDown),
            command_description: "Preset Down".into(),
            command: ShortcutCommand::UiPayload(UiPayload::DeviceState(DeviceState::DeltaPreset { delta: -1 }))
        }
    ];

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
    
    s
}