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