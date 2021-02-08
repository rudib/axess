use nwd::NwgUi;
use nwg::{CheckBoxState, ComboBox, NativeUi};

use std::{cell::RefCell, sync::{Mutex, Arc}};

use axess_core::{payload::{PayloadConnection, UiPayload}, transport::Endpoint};
use super::{common::{FractalWindow, WindowApi}, keyboard::UiEvent};
use crate::windows::tuner::tuner_window_ui::TunerWindowUi;

#[derive(NwgUi, Default)]
pub struct TunerWindow {
    #[nwg_control(size: (300, 250), title: "Tuner", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [TunerWindow::on_exit], OnInit: [TunerWindow::init] )]
    window: nwg::Window,

    #[nwg_control]
    #[nwg_events( OnNotice: [TunerWindow::backend_response] )]
    backend_response_notifier: nwg::Notice,

    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,

    ui_api: Option<WindowApi>
}

impl FractalWindow for TunerWindow {
    type WindowUi = TunerWindowUi;
    type Window = TunerWindow;
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

    fn handle_ui_event(&self, _event: UiEvent) -> bool {
        true
    }
}

impl TunerWindow {
    fn init(&self) {
        /*
        if let Ok(config) = self.get_window_api_initialized().config.lock() {
            self.checkbox_keys_axe.set_check_state(if config.keyboard_shortcuts_axe_edit { CheckBoxState::Checked } else { CheckBoxState::Unchecked });
            self.checkbox_keys_fn.set_check_state(if config.keyboard_shortcuts_presets_and_scenes_function_keys { CheckBoxState::Checked } else { CheckBoxState::Unchecked });
        }
        */
        self.send(UiPayload::EnableTuner);
    }

    fn on_exit(&self) {
        self.send(UiPayload::DisableTuner);
    }

    fn cancel(&self) {
        self.on_exit();
    }

    fn backend_response(&self) {
        let _msg = self.recv();
    }
}