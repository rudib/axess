use nwd::NwgUi;

use std::{cell::RefCell};

use axess_core::{payload::{PayloadConnection, UiPayload, DeviceState, PresetAndScene}};
use super::{common::{FractalWindow, WindowApi}, connect::ConnectWindow};
use crate::windows::main::main_window_ui::MainWindowUi;

const NOT_CONNECTED: &'static str = "Not connected.";

#[derive(NwgUi, Default)]
pub struct MainWindow {
    #[nwg_control(title: "Axess Fractal Audio Editor", flags: "MAIN_WINDOW|VISIBLE")]
    #[nwg_events( OnInit: [MainWindow::init], OnWindowClose: [MainWindow::on_exit], OnKeyPress: [MainWindow::on_key_press(SELF, EVT_DATA)] )]
    window: nwg::Window,

    #[nwg_control(text: "&Device")]
    menu_device: nwg::Menu,

    #[nwg_control(text: "Connect", parent: menu_device)]
    #[nwg_events( OnMenuItemSelected: [MainWindow::connect] )]
    menu_device_connect: nwg::MenuItem,

    #[nwg_control(text: "Disconnect", parent: menu_device, disabled: true)]
    #[nwg_events( OnMenuItemSelected: [MainWindow::disconnect] )]
    menu_device_disconnect: nwg::MenuItem,

    #[nwg_control(parent: menu_device)]
    menu_device_sep: nwg::MenuSeparator,

    #[nwg_control(text: "Exit", parent: menu_device)]
    #[nwg_events( OnMenuItemSelected: [MainWindow::on_exit] )]
    menu_device_exit: nwg::MenuItem,


    #[nwg_control(text: "&Help")]
    menu_help: nwg::Menu,

    #[nwg_control(text: "About", parent: menu_help)]
    menu_help_about: nwg::MenuItem,


    #[nwg_control(text: NOT_CONNECTED, parent: window)]
    status_bar: nwg::StatusBar,



    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,

    #[nwg_control]
    #[nwg_layout_item(layout: grid, row: 0, col: 0)]
    preset_number: nwg::Label,

    #[nwg_control]
    #[nwg_layout_item(layout: grid, row: 0, col: 1)]
    preset_name: nwg::Label,

    #[nwg_control(text: "Previous Preset")]
    #[nwg_layout_item(layout: grid, row: 1, col: 0)]
    #[nwg_events(OnButtonClick: [MainWindow::previous_preset])]
    preset_minus: nwg::Button,
    
    #[nwg_control(text: "Next Preset")]
    #[nwg_layout_item(layout: grid, row: 1, col: 1)]
    #[nwg_events(OnButtonClick: [MainWindow::next_preset])]
    preset_plus: nwg::Button,

    #[nwg_control]
    #[nwg_layout_item(layout: grid, row: 2, col: 0)]
    scene_number: nwg::Label,

    #[nwg_control]
    #[nwg_layout_item(layout: grid, row: 2, col: 1)]
    scene_name: nwg::Label,

    #[nwg_control(text: "Previous Scene")]
    #[nwg_layout_item(layout: grid, row: 3, col: 0)]
    #[nwg_events(OnButtonClick: [MainWindow::previous_scene])]
    scene_minus: nwg::Button,
    
    #[nwg_control(text: "Next Scene")]
    #[nwg_layout_item(layout: grid, row: 3, col: 1)]
    #[nwg_events(OnButtonClick: [MainWindow::next_scene])]
    scene_plus: nwg::Button,

    #[nwg_control]
    #[nwg_events( OnNotice: [MainWindow::backend_response] )]
    backend_response_notifier: nwg::Notice,

    pub ui_api: Option<WindowApi>,

    pub device_state: RefCell<Option<PresetAndScene>>,
    pub is_connected: RefCell<bool>
}

impl FractalWindow for MainWindow {
    type Data = ();
    type WindowUi = MainWindowUi;
    type Window = MainWindow;

    fn set_window_api(&mut self, api: WindowApi) {
        self.ui_api = Some(api);
    }

    fn get_window_api(&self) -> &Option<WindowApi> {
        &self.ui_api
    }

    fn get_notice(&self) -> &nwg::Notice {
        &self.backend_response_notifier
    }
}

impl MainWindow {
    fn main_controls_when_connected(&self, visibility: bool) {
        self.preset_number.set_visible(visibility);
        self.preset_name.set_visible(visibility);
        self.preset_minus.set_visible(visibility);
        self.preset_plus.set_visible(visibility);
        self.scene_number.set_visible(visibility);
        self.scene_name.set_visible(visibility);
        self.scene_plus.set_visible(visibility);
        self.scene_minus.set_visible(visibility);
    }

    fn init(&self) {
        self.main_controls_when_connected(false);
        self.send(UiPayload::Connection(PayloadConnection::TryToAutoConnect));        
    }

    fn connect(&self) {
        // todo: check if we can auto connect, without a window
        //self.send(UiPayload::Connection(PayloadConnection::TryToAutoConnect));
        self.spawn_child::<ConnectWindow>(());
    }

    fn disconnect(&self) {
        self.send(UiPayload::Connection(PayloadConnection::Disconnect));
    }

    fn backend_response(&self) {
        match self.recv() {
            Some(UiPayload::Connection(PayloadConnection::AutoConnectDeviceNotFound)) => {
                // start the connect window
                //self.spawn_child::<ConnectWindow>(());
            },
            Some(UiPayload::Connection(PayloadConnection::Connected { ref device })) => {
                self.main_controls_when_connected(true);
                self.status_bar.set_text(0, &format!("Connected to {}.", device));
                self.menu_device_connect.set_enabled(false);
                self.menu_device_disconnect.set_enabled(true);   
                *self.is_connected.borrow_mut() = true;
            },
            Some(UiPayload::Connection(PayloadConnection::Disconnect)) => {
                self.main_controls_when_connected(false);
                self.status_bar.set_text(0, NOT_CONNECTED);
                self.menu_device_connect.set_enabled(true);
                self.menu_device_disconnect.set_enabled(false);
                *self.is_connected.borrow_mut() = false;
            },
            Some(UiPayload::DeviceState(DeviceState::PresetAndScene(ref p))) => {
                self.preset_number.set_text(&format!("{:0>3}", p.preset));
                self.preset_name.set_text(&p.preset_name);
                self.scene_number.set_text(&format!("Scene {}", p.scene + 1));
                self.scene_name.set_text(&p.scene_name);

                *self.device_state.borrow_mut() = Some(p.clone());
            },
            Some(_) => {}
            None => {}
        }
    }

    fn preset_delta(&self, delta: i16) {
        if let Some(ref device_state) = *self.device_state.borrow() {

            let mut p = device_state.preset as i16 + delta;
            if p < 0 { p = 511; }
            if p > 511 { p = 0; }

            self.send(UiPayload::DeviceState(DeviceState::SetPreset { preset: p as u16 }));
        }
    }

    fn previous_preset(&self) {
        self.preset_delta(-1);
    }

    fn next_preset(&self) {
        self.preset_delta(1);
    }

    fn scene_delta(&self, delta: i8) {
        if let Some(ref device_state) = *self.device_state.borrow() {

            let mut s = device_state.scene as i8 + delta;
            if s < 0 { s = 7; }
            if s > 7 { s = 0; }

            self.send(UiPayload::DeviceState(DeviceState::SetScene { scene: s as u8 }));
        }
    }

    fn previous_scene(&self) {
        self.scene_delta(-1);
    }

    fn next_scene(&self) {
        self.scene_delta(1);
    }

    fn on_key_press(&self, data: &nwg::EventData) {
        if *self.is_connected.borrow() == false { return; }

        if let nwg::EventData::OnKey(key) = data {
            if *key == 'W' as u32 {
                self.previous_scene();
            } else if *key == 'S' as u32 {
                self.next_scene();
            } else if *key == 'D' as u32 {
                self.next_preset();
            } else if *key == 'A' as u32 {
                self.previous_preset();
            }
        }
    }
}