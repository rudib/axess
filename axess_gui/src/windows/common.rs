use std::{thread, cell::RefCell, sync::{Mutex, Arc}, ops::Deref, any::type_name};
use axess_core::{payload::UiPayload, backend::UiApi};
use futures::{executor::block_on, future::Either};
use nwg::{NativeUi};
use log::trace;

use crate::config::AxessConfiguration;

use super::keyboard::UiEvent;

pub trait FractalWindow {
    type Data;
    type WindowUi: Deref<Target=Self::Window>;
    type Window: Default + NativeUi<Self::WindowUi> + FractalWindow;

    fn set_window_api(&mut self, api: WindowApi);
    fn get_window_api(&self) -> &Option<WindowApi>;
    fn get_notice(&self) -> &nwg::Notice;
    fn handle_ui_event(&self, event: UiEvent) -> bool;

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

    fn spawn(_data: Self::Data, api: WindowApi) -> std::result::Result<std::thread::JoinHandle<()>, std::io::Error> {
        thread::Builder::new()
            .name(format!("{} Main Thread", type_name::<Self::Window>()))
            .spawn(move || {
            let mut window_data = Self::Window::default();
            window_data.set_window_api(api.clone());
            
            let window = Self::Window::build_ui(window_data).expect("Failed to build UI");
            let notice_sender = window.deref().get_notice().sender();
            
            let (stop_sender, mut stop_receiver) = futures::channel::oneshot::channel::<()>();

            // message notifier
            {
                let api = api.clone();
                thread::Builder::new()
                    .name(format!("{} Message Pump", type_name::<Self::Window>()))
                    .spawn(move || {

                        block_on(async {
                            let mut api = api.api.borrow_mut();
                            loop {                                
                                let msg_fut = api.channel.recv();

                                match futures::future::select(msg_fut, &mut stop_receiver).await {
                                    Either::Left(_) => notice_sender.notice(),
                                    Either::Right(_) => { break; }
                                }
                            }
                        });

                    drop(api);
                    trace!("Stopped thread for Window {}", type_name::<Self::Window>());
                }).unwrap();
            }

            {
                nwg::dispatch_thread_events_with_pretranslate(move |m| {
                    if m.message == 0x0100 {
                        return window.handle_ui_event(UiEvent::KeyDown(m.wParam, m.lParam as u32));
                    } else if m.message == 0x0101 {
                        return window.handle_ui_event(UiEvent::KeyUp(m.wParam, m.lParam as u32));
                    }

                    return true;
                });
            }
            
            // ..and we are shutting down the window
            stop_sender.send(()).expect("stop sender failure");
            
            // todo: this is api #3 with a queue... somehow get only the sender?
            match block_on(api.api.borrow().channel.send(&UiPayload::Ping)) {
                Err(e) => trace!("Failed to send final ping: {:?}", e),
                _ => ()
            };
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
    api: RefCell<UiApi>,
    pub config: RefCell<AxessConfiguration>
}

impl WindowApi {
    pub fn new(api: UiApi, config: AxessConfiguration) -> Self {
        WindowApi {
            api: RefCell::new(api),
            config: RefCell::new(config)
        }
    }
}