#[derive(Default, Clone)]
pub struct AxessStatusBar {
    messages: Vec<(AxessStatusBarMessageKind, String)>
}

impl AxessStatusBar {
    pub fn op<'a, 'b>(&'a mut self, status_bar: &'b nwg::StatusBar) -> AxessStatusBarOperation<'a, 'b> {
        AxessStatusBarOperation {
            s: self,
            status_bar
        }
    }
}

pub struct AxessStatusBarOperation<'a, 'b> {
    s: &'a mut AxessStatusBar,
    status_bar: &'b nwg::StatusBar
}

impl<'a, 'b> AxessStatusBarOperation<'a, 'b> {
    pub fn push_message(&mut self, message_kind: AxessStatusBarMessageKind, message: String) {
        let existing = self.s.messages.iter_mut().find(|p| p.0 == message_kind);
        if let Some(existing) = existing {
            existing.1 = message;
        } else {
            self.s.messages.push((message_kind, message));
        }
    }

    pub fn pop_message(&mut self, message_kind: AxessStatusBarMessageKind) {
        self.s.messages.retain(|m| m.0 != message_kind);
    }
}

impl<'a, 'b> Drop for AxessStatusBarOperation<'a, 'b> {
    fn drop(&mut self) {
        if let Some(msg) = self.s.messages.last() {
            self.status_bar.set_text(0, &msg.1);
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AxessStatusBarMessageKind {
    Default,
    Connected,
    Progress
}