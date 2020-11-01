use nwd::NwgUi;
use nwg::{ComboBox, NativeUi};

use std::{cell::RefCell, sync::{Mutex, Arc}};

use axess_core::{payload::{PayloadConnection, UiPayload}, transport::Endpoint};
use super::{common::{FractalWindow, WindowApi}, keyboard::UiEvent};
use crate::windows::connect::connect_window_ui::ConnectWindowUi;

#[derive(NwgUi, Default)]
pub struct ConnectWindow {
    #[nwg_control(size: (300, 250), title: "Connect to a Fractal Audio device", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [ConnectWindow::on_exit], OnInit: [ConnectWindow::init] )]
    window: nwg::Window,

    #[nwg_control]
    #[nwg_events( OnNotice: [ConnectWindow::backend_response] )]
    backend_response_notifier: nwg::Notice,


    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,


    #[nwg_control(text: "Port", h_align: HTextAlign::Left)]
    #[nwg_layout_item(layout: grid, row: 0, col: 0)]
    label_port: nwg::Label,

    #[nwg_control()]
    #[nwg_layout_item(layout: grid, row: 1, col: 0)]
    port: nwg::ComboBox<String>,

    #[nwg_control(text: "&Connect")]
    #[nwg_layout_item(layout: grid, row: 2, col: 0)]
    #[nwg_events( OnButtonClick: [ConnectWindow::connect] )]
    connect_button: nwg::Button,

    ui_api: Option<WindowApi>,
    endpoints: RefCell<Vec<Endpoint>>
}

impl FractalWindow for ConnectWindow {
    type WindowUi = ConnectWindowUi;
    type Window = ConnectWindow;
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


impl ConnectWindow {
    fn connect(&self) {        
        if let Some(idx) = self.port.selection() {
            if let Some(endpoint) = self.endpoints.borrow().get(idx) {
                self.send(UiPayload::Connection(PayloadConnection::ConnectToEndpoint(endpoint.clone())));
            }
        }
    }

    fn init(&self) {
        self.send(UiPayload::Connection(PayloadConnection::ListEndpoints))
    }
    
    fn backend_response(&self) {
        // there should be a message waiting for
        // read without locking, apply the UI changes        
        let msg = self.recv();
   
        match msg {
            Some(UiPayload::Connection(PayloadConnection::DetectedEndpoints { endpoints })) => {

                let len = endpoints.len();
                if len == 0 {
                    self.port.set_collection(vec!["None found!".to_string()]);
                    //dropdown.set_selection(Some(0));
                    self.port.set_enabled(false);
                } else {
                    self.port.set_enabled(true);
                    self.port.set_collection(endpoints.iter().map(|e| e.transport_endpoint.name.clone()).collect());
                    self.port.set_selection(Some(0));
                }

                *self.endpoints.borrow_mut() = endpoints;
            },
            Some(UiPayload::Connection(PayloadConnection::Connected { .. })) => {
                self.on_exit();
            }
            Some(_) => {}
            None => {}
        }
    }
}
