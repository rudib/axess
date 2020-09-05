use nwd::NwgUi;
use nwg::NativeUi;

use std::thread;
use std::{cell::RefCell};

use fractal_backend::{UiApi, UiBackend, UiPayload, PayloadConnection};
use super::{common::{FractalWindow, WindowApi}, connect::ConnectWindow};
use crate::windows::main::main_window_ui::MainWindowUi;

const NOT_CONNECTED: &'static str = "Not connected.";

#[derive(NwgUi, Default)]
pub struct MainWindow {
    #[nwg_control(title: "Axess Fractal Audio Editor", flags: "MAIN_WINDOW|VISIBLE")]
    #[nwg_events( OnInit: [MainWindow::init], OnWindowClose: [MainWindow::on_exit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,

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


    #[nwg_control]
    #[nwg_events( OnNotice: [MainWindow::backend_response] )]
    backend_response_notifier: nwg::Notice,

    pub ui_api: Option<WindowApi>
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
    fn init(&self) {
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
                self.status_bar.set_text(0, &format!("Connected to {:?}", device));
                self.menu_device_connect.set_enabled(false);
                self.menu_device_disconnect.set_enabled(true);                
            },
            Some(UiPayload::Connection(PayloadConnection::Disconnect)) => {
                self.status_bar.set_text(0, NOT_CONNECTED);
                self.menu_device_connect.set_enabled(true);
                self.menu_device_disconnect.set_enabled(false);
            },
            Some(_) => {}
            None => {}
        }
    }
}