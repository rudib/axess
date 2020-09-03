extern crate fractal_protocol;
extern crate fractal_core;
extern crate broadcaster;

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

use std::thread;
use std::{cell::RefCell};

use fractal_backend::{UiApi, UiBackend, UiPayload};
use futures::executor::block_on;
//use bus::BusReader;

#[derive(NwgUi)]
pub struct ConnectWindow {
    #[nwg_control(size: (300, 115), position: (300, 300), title: "Connect to a Fractal Audio device", flags: "WINDOW|VISIBLE")]
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

    ui_api: RefCell<UiApi>
}

impl ConnectWindow {

    fn init(&self) {
        let ref mut ui_api = self.ui_api.borrow_mut();
        block_on(ui_api.channel.send(&UiPayload::ListMidiPorts)).unwrap();
    }
    
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn backend_response(&self) {
        println!("now what?");

        // there should be a message waiting for
        // read without locking, apply the UI changes
        
        let msg = {
            block_on(self.ui_api.borrow_mut().channel.recv())
        };
   
        match msg {
            Some(UiPayload::DetectedMidiPorts { ports }) => {
                
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
            },
            Some(_) => {}
            None => {}
        }
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let mut ui_api = UiBackend::spawn();

    let connect_window = ConnectWindow {
        ui_api: RefCell::new(ui_api.clone()),
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
            if let Some(_) = block_on(ui_api.channel.recv()) {
                notice_sender.notice();
            } else {
                break;
            }
        }

        println!("pump out!");
    });

    nwg::dispatch_thread_events();
}