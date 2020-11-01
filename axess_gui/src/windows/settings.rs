use nwd::NwgUi;
use nwg::{CheckBoxState, ComboBox, NativeUi};

use std::{cell::RefCell, sync::{Mutex, Arc}};

use axess_core::{payload::{PayloadConnection, UiPayload}, transport::Endpoint};
use super::{common::{FractalWindow, WindowApi}, keyboard::UiEvent};
use crate::windows::settings::settings_window_ui::SettingsWindowUi;

#[derive(NwgUi, Default)]
pub struct SettingsWindow {
    #[nwg_control(size: (300, 250), title: "Axess Settings", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [SettingsWindow::on_exit], OnInit: [SettingsWindow::init] )]
    window: nwg::Window,

    #[nwg_control]
    #[nwg_events( OnNotice: [SettingsWindow::backend_response] )]
    backend_response_notifier: nwg::Notice,


    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,

    #[nwg_control(text: "Axe-Edit keyboard shortcuts")]
    #[nwg_layout_item(layout: grid, row: 0, col: 0, col_span: 2)]
    checkbox_keys_axe: nwg::CheckBox,

    #[nwg_control(text: "Function keys shortcuts")]
    #[nwg_layout_item(layout: grid, row: 1, col: 0, col_span: 2)]
    checkbox_keys_fn: nwg::CheckBox,

    #[nwg_control(text: "&Save")]
    #[nwg_layout_item(layout: grid, row: 2, col: 0)]
    #[nwg_events( OnButtonClick: [SettingsWindow::save] )]
    save_button: nwg::Button,

    #[nwg_control(text: "&Cancel")]
    #[nwg_layout_item(layout: grid, row: 2, col: 1)]
    #[nwg_events( OnButtonClick: [SettingsWindow::cancel] )]
    cancel_button: nwg::Button,

    ui_api: Option<WindowApi>
}

impl FractalWindow for SettingsWindow {
    type WindowUi = SettingsWindowUi;
    type Window = SettingsWindow;
    type Data = ();
    
    fn set_window_api(&mut self, api: WindowApi) {
        self.ui_api = Some(api);
    }

    fn get_window_api(&self) -> &Option<WindowApi> {
        &self.ui_api
    }

    fn get_notice(&self) -> &nwg::Notice {
        &self.backend_response_notifier
    }

    fn handle_ui_event(&self, event: UiEvent) -> bool {
        true
    }
}

impl SettingsWindow {
    fn init(&self) {
        let config = self.get_window_api_initialized().config.borrow();
        self.checkbox_keys_axe.set_check_state(if config.keyboard_shortcuts_axe_edit { CheckBoxState::Checked } else { CheckBoxState::Unchecked });
        self.checkbox_keys_fn.set_check_state(if config.keyboard_shortcuts_presets_and_scenes_function_keys { CheckBoxState::Checked } else { CheckBoxState::Unchecked });
    }

    fn save(&self) {
        let mut config = self.get_window_api_initialized().config.borrow_mut();
        config.keyboard_shortcuts_axe_edit = if self.checkbox_keys_axe.check_state() == CheckBoxState::Checked { true } else { false };
        config.keyboard_shortcuts_presets_and_scenes_function_keys = if self.checkbox_keys_fn.check_state() == CheckBoxState::Checked { true } else { false };
        config.save();

        self.on_exit();
    }

    fn cancel(&self) {
        self.on_exit();
    }

    fn backend_response(&self) {
        let _msg = self.recv();
    }
}