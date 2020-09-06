use std::{thread, cell::RefCell, sync::{Mutex, Arc}, ops::Deref, any::type_name};
use axess_core::{payload::UiPayload, backend::UiApi};
use futures::executor::block_on;
use nwg::{NwgError, NativeUi};

pub trait FractalWindow {
    type Data;
    type WindowUi: Deref<Target=Self::Window>;
    type Window: Default + NativeUi<Self::WindowUi> + FractalWindow;

    fn set_window_api(&mut self, api: WindowApi);
    fn get_window_api(&self) -> &Option<WindowApi>;
    fn get_notice(&self) -> &nwg::Notice;

    fn send(&self, payload: UiPayload) {
        let window_api = self.get_window_api_initialized();
        let api = window_api.api.borrow();
        block_on(api.channel.send(&payload)).unwrap();
    }

    fn recv(&self) -> Option<UiPayload> {        
        let window_api = self.get_window_api_initialized();
        let mut api = window_api.api.borrow_mut();
        block_on(api.channel.recv())
    }

    fn get_window_api_initialized(&self) -> &WindowApi {
        self.get_window_api().as_ref().expect("should be initialized")
    }

    fn spawn(data: Self::Data, api: WindowApi) -> std::result::Result<std::thread::JoinHandle<()>, std::io::Error> {
        thread::Builder::new()
            .name(format!("{} Main Thread", type_name::<Self::Window>()))
            .spawn(move || {
            let mut window_data = Self::Window::default();
            window_data.set_window_api(api.clone());
            
            let window = Self::Window::build_ui(window_data).expect("Failed to build UI");
            let notice_sender = window.deref().get_notice().sender();
            
            let stop = Arc::new(Mutex::new(false));

            // message notifier
            {
                let api = api.clone();
                let stop = stop.clone();
                thread::Builder::new()
                    .name(format!("{} Message Pump", type_name::<Self::Window>()))
                    .spawn(move || {
                    loop {
                        // todo: replace "stop" with a multi-select
                        let msg = block_on(api.api.borrow_mut().channel.recv());

                        if let Some(_) = msg {
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
                    drop(api);
                }).unwrap();
            }

            nwg::dispatch_thread_events();
            
            if let Ok(mut stop) = stop.lock() {
                *stop = true;
            }
            
            // todo: this is api #3 with a queue... somehow get only the sender?
            block_on(api.api.borrow().channel.send(&UiPayload::Ping)).unwrap();
        })
    }

    fn spawn_child<T: FractalWindow>(&self, data: T::Data) {
        let api = self.get_window_api_initialized().clone();
        T::spawn(data, api).unwrap();
    } 

    fn on_exit(&self) {
        nwg::stop_thread_dispatch();
    }
}
#[derive(Clone)]
pub struct WindowApi {
    api: RefCell<UiApi>
}

impl WindowApi {
    pub fn new(api: UiApi) -> Self {
        WindowApi {
            api: RefCell::new(api)
        }
    }
}