use nwd::NwgUi;
use nwg::NativeUi;

use std::thread;
use std::{cell::RefCell};

use fractal_backend::{UiApi, UiBackend, UiPayload};
use futures::executor::block_on;
use super::connect::ConnectWindow;

#[derive(NwgUi, Default)]
pub struct MainWindow {
    #[nwg_control(title: "Axess Fractal Audio Editor", flags: "MAIN_WINDOW|VISIBLE")]
    //#[nwg_events( OnWindowClose: [ConnectWindow::exit], OnInit: [ConnectWindow::init] )]
    #[nwg_events( OnInit: [MainWindow::init], OnWindowClose: [MainWindow::exit] )]
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

    #[nwg_control(text: "&Help")]
    menu_help: nwg::Menu,

    /*
    #[nwg_control(text: "MIDI Input", h_align: HTextAlign::Left)]
    #[nwg_layout_item(layout: grid, row: 0, col: 0)]
    label_midi_input: nwg::Label,

    #[nwg_control()]
    #[nwg_layout_item(layout: grid, row: 1, col: 0)]
    midi_input: nwg::ComboBox<String>,
    */

    #[nwg_control]
    #[nwg_events( OnNotice: [MainWindow::backend_response] )]
    backend_response_notifier: nwg::Notice,

    /*
    #[nwg_control(text: "Heisenberg", focus: true)]
    #[nwg_layout_item(layout: grid, row: 0, col: 0)]
    name_edit: nwg::TextInput,

    #[nwg_control(text: "Say my name")]
    #[nwg_layout_item(layout: grid, col: 0, row: 1, row_span: 2)]
    #[nwg_events( OnButtonClick: [BasicApp::say_hello] )]
    hello_button: nwg::Button
    */

    pub ui_api: Option<RefCell<UiApi>>
}

impl MainWindow {
    fn init(&self) {

    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn connect(&self) {
        // open the window?
        //println!("connect?");
        let api = self.ui_api.as_ref().unwrap().borrow_mut().clone();
        ConnectWindow::spawn(api);
    }

    fn disconnect(&self) {
        println!("disconnect?");
    }

    fn backend_response(&self) {

    }
}