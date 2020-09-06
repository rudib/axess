use nwd::NwgUi;
use nwg::{ComboBox, NativeUi};

use std::{cell::RefCell, sync::{Mutex, Arc}};

use fractal_backend::{payload::{PayloadConnection, UiPayload, ConnectToMidiPorts}};
use super::common::{FractalWindow, WindowApi};
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


    #[nwg_control(text: "MIDI Input", h_align: HTextAlign::Left)]
    #[nwg_layout_item(layout: grid, row: 0, col: 0)]
    label_midi_input: nwg::Label,

    #[nwg_control()]
    #[nwg_layout_item(layout: grid, row: 1, col: 0)]
    midi_input: nwg::ComboBox<String>,

    #[nwg_control(text: "MIDI Output", h_align: HTextAlign::Left)]
    #[nwg_layout_item(layout: grid, row: 2, col: 0)]
    label_midi_output: nwg::Label,

    #[nwg_control()]
    #[nwg_layout_item(layout: grid, row: 3, col: 0)]
    midi_output: nwg::ComboBox<String>,


    #[nwg_control(text: "&Connect")]
    #[nwg_layout_item(layout: grid, row: 4, col: 0, row_span: 2)]
    #[nwg_events( OnButtonClick: [ConnectWindow::connect] )]
    connect_button: nwg::Button,

    ui_api: Option<WindowApi>
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
}


impl ConnectWindow {
    fn connect(&self) {        
        self.send(UiPayload::Connection(PayloadConnection::ConnectToMidiPorts(ConnectToMidiPorts {
            input_port: self.midi_input.selection_string().unwrap(),
            output_port: self.midi_output.selection_string().unwrap()
        })))
    }

    fn init(&self) {
        self.send(UiPayload::Connection(PayloadConnection::ListMidiPorts))
    }
    
    fn backend_response(&self) {
        // there should be a message waiting for
        // read without locking, apply the UI changes        
        let msg = self.recv();
   
        match msg {
            Some(UiPayload::Connection(PayloadConnection::DetectedMidiPorts { ports })) => {
                
                let set_ports = |dropdown: &ComboBox<String>, ports: &Vec<String>| {
                    let len = ports.len();
                    if len == 0 {
                        dropdown.set_collection(vec!["None found!".to_string()]);
                        //dropdown.set_selection(Some(0));
                        dropdown.set_enabled(false);
                    } else {
                        dropdown.set_enabled(true);
                        dropdown.set_collection(ports.to_vec());
                        dropdown.set_selection(Some(0));
                    }
                };
                
                set_ports(&self.midi_input, &ports.inputs);
                set_ports(&self.midi_output, &ports.outputs);

                let detected = ports.detect_fractal_devices();
                if let Some(device ) = detected.first() {
                    self.midi_input.set_selection_string(&device.input_port_name);
                    self.midi_output.set_selection_string(&device.output_port_name);
                }

            },
            Some(UiPayload::Connection(PayloadConnection::Connected { .. })) => {
                self.on_exit();
            }
            Some(_) => {}
            None => {}
        }
    }
}
