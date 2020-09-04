use nwd::NwgUi;
use nwg::{ComboBox, NativeUi};

use std::thread;
use std::{cell::RefCell, sync::{Mutex, Arc}};

use fractal_backend::{UiApi, UiBackend, UiPayload};
use futures::executor::block_on;

#[derive(NwgUi, Default)]
pub struct ConnectWindow {
    #[nwg_control(size: (300, 250), title: "Connect to a Fractal Audio device", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [ConnectWindow::exit], OnInit: [ConnectWindow::init] )]
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

    ui_api: Option<RefCell<UiApi>>
}

impl ConnectWindow {

    pub fn spawn(mut api: UiApi) {
        thread::spawn(|| {
            let mut window_data = Self::default();
            window_data.ui_api = Some(RefCell::new(api.clone()));

            let window = Self::build_ui(window_data).expect("Failed to build UI");
            let notice_sender = window.backend_response_notifier.sender();
            
            let stop = Arc::new(Mutex::new(false));

            // message notifier
            {
                let stop = stop.clone();
                thread::spawn(move || {
                    loop {
                        if let Some(_) = block_on(api.channel.recv()) {
                            notice_sender.notice();
                        } else {
                            break;
                        }

                        if let Ok(stop) = stop.lock() {
                            if *stop == true {
                                break;
                            }
                        }
                    }
                    println!("stop 2");
                });
            }

            nwg::dispatch_thread_events();
            
            if let Ok(mut stop) = stop.lock() {
                *stop = true;
            }
            block_on(window.ui_api.as_ref().unwrap().borrow_mut().channel.send(&UiPayload::Ping)).unwrap();
            println!("stop 1");
        });
    }

    fn connect(&self) {
        //println!("connect?");
        let ref mut ui_api = self.ui_api.as_ref().unwrap().borrow_mut();
        block_on(ui_api.channel.send(&UiPayload::ConnectToMidiPorts {
            input_port: self.midi_input.selection_string().unwrap(),
            output_port: self.midi_output.selection_string().unwrap()
        })).unwrap();
    }

    fn init(&self) {
        let ref mut ui_api = self.ui_api.as_ref().unwrap().borrow_mut();
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
            block_on(self.ui_api.as_ref().unwrap().borrow_mut().channel.recv())
        };
   
        match msg {
            Some(UiPayload::DetectedMidiPorts { ports }) => {
                
                let set_ports = |dropdown: &ComboBox<String>, ports: &Vec<String>| {
                    let len = ports.len();
                    if len == 0 {
                        dropdown.set_collection(vec!["None found!".to_string()]);
                        dropdown.set_selection(Some(0));
                        dropdown.set_enabled(false);
                    } else {
                        dropdown.set_enabled(true);
                        dropdown.set_collection(ports.to_vec());
                        dropdown.set_selection(Some(0));
                    }
                };
                
                set_ports(&self.midi_input, &ports.inputs);
                set_ports(&self.midi_output, &ports.outputs);

            },
            Some(_) => {}
            None => {}
        }
    }
}
