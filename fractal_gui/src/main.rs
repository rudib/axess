extern crate fractal_protocol;
extern crate fractal_core;
extern crate bus;


extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

use std::thread;
use std::{cell::RefCell, sync::{Mutex, Arc}};

use fractal_core::midi::*;
use fractal_backend::{UiApi, UiBackend, UiRequest, UiCommand, UiResponse};
use bus::BusReader;

#[derive(NwgUi)]
pub struct ConnectWindow {
    #[nwg_control(size: (300, 115), position: (300, 300), title: "Connect to a Fractal Audio device", flags: "WINDOW|VISIBLE")]
    //#[nwg_events( OnWindowClose: [BasicApp::say_goodbye] )]
    #[nwg_events( OnWindowClose: [ConnectWindow::exit], OnInit: [ConnectWindow::init] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,


    #[nwg_control(text: "MIDI Input", h_align: HTextAlign::Left)]
    #[nwg_layout_item(layout: grid, row: 0, col: 0)]
    label_midi_input: nwg::Label,

    #[nwg_control()]
    #[nwg_layout_item(layout: grid, row: 1, col: 0)]
    midi_input: nwg::ComboBox<String>,

    #[nwg_control]
    #[nwg_events( OnNotice: [ConnectWindow::backend_response] )]
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

    ui_api: RefCell<UiApi>,
    ui_api_reader: RefCell<BusReader<UiResponse>>
}

impl ConnectWindow {

    fn init(&self) {
        println!("heyo?");
        self.ui_api.borrow_mut().input.broadcast(UiRequest::new(UiCommand::ListMidiPorts));
    }
    
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn backend_response(&self) {
        println!("now what?");
        // there should be a message waiting for
        // read without locking, apply the UI changes
        if let Ok(response) = self.ui_api_reader.borrow_mut().try_recv() {
            match response.payload {
                fractal_backend::UiPayload::DetectedMidiPorts { ports } => {
                    
                    println!("setting ports!");
                    
                    let len = ports.inputs.len();                    
                    if len == 0 {
                        // notify that none found?
                        self.midi_input.set_collection(vec!["None found!".to_string()]);
                        self.midi_input.set_selection(Some(0));
                        self.midi_input.set_enabled(false);
                    } else {
                        self.midi_input.set_enabled(true);
                        self.midi_input.set_collection(ports.inputs);
                    }

                }
            }
        }
    }

    /*
    fn say_hello(&self) {
        nwg::modal_info_message(&self.window, "Hello", &format!("Hello {}", self.name_edit.text()));
    }
    
    fn say_goodbye(&self) {
        nwg::modal_info_message(&self.window, "Goodbye", &format!("Goodbye {}", self.name_edit.text()));
        nwg::stop_thread_dispatch();
    }
    */
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let mut backend = UiBackend::spawn();
    let mut rx_pump = backend.new_response_reader();
    let rx_ui = backend.new_response_reader();

    let connect_window = ConnectWindow {
        ui_api: RefCell::new(backend),
        ui_api_reader: RefCell::new(rx_ui),

        window: Default::default(),
        grid: Default::default(),
        label_midi_input: Default::default(),
        midi_input: Default::default(),
        backend_response_notifier: Default::default()
    };

    let app: connect_window_ui::ConnectWindowUi = ConnectWindow::build_ui(connect_window).expect("Failed to build UI");
    
    let notice_sender = app.backend_response_notifier.sender();
    
    // backend msg notifier pump
    thread::spawn(move || {        
        loop {
            if let Ok(_) = rx_pump.recv() {
                notice_sender.notice();
            } else {
                break;
            }
        }

        println!("pump out!");
    });

    nwg::dispatch_thread_events();
}



/*
fn main() {
    let midi = Midi::new();
    let midi_ports = midi.detect_midi_ports().unwrap();
    println!("all midi ports: {:?}", midi_ports);
    let fractals = midi_ports.detect_fractal_devices();
    println!("fractals: {:?}", fractals);


}
*/