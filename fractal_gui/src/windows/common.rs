use std::{thread, cell::RefCell, sync::{Mutex, Arc}, ops::Deref};
use fractal_backend::{UiPayload, UiApi};
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
        let api = window_api.api.lock().unwrap();
        block_on(api.channel.send(&payload)).unwrap();
    }

    fn recv(&self) -> Option<UiPayload> {
        let window_api = self.get_window_api_initialized();
        block_on(window_api.api.lock().unwrap().channel.recv())
    }

    fn get_window_api_initialized(&self) -> &WindowApi {
        self.get_window_api().as_ref().expect("should be initialized")
    }

    //fn build_window(data: Self) -> Result<Self::WindowUi, NwgError>;

    fn spawn(data: Self::Data, api: WindowApi) {
        thread::spawn(move || {
            let mut window_data = Self::Window::default();
            window_data.set_window_api(api.clone());

            //let window = Self::build_ui(window_data).expect("Failed to build UI");
            let window = Self::Window::build_ui(window_data).expect("Failed to build UI");
            let notice_sender = window.deref().get_notice().sender();
            
            let stop = Arc::new(Mutex::new(false));

            // message notifier
            {
                let api = api.clone();
                let stop = stop.clone();
                thread::spawn(move || {
                    println!("s1");    
                    loop {
                        println!("s2");
                        //let msg = block_on(api.channel.recv());

                        if let Ok(ref mut api) = api.api.lock() {
                            println!("s3");
                            let msg = block_on(api.channel.recv());
                            if let Some(_) = msg {
                                println!("got msg?");
                                notice_sender.notice();
                            } else {
                                break;
                            }
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
            //block_on(window.ui_api.as_ref().unwrap().borrow_mut().channel.send(&UiPayload::Ping)).unwrap();
            println!("stop 1");
        });
    }

    fn spawn_child<T: FractalWindow>(&self, data: T::Data) {
        let api = self.get_window_api_initialized().clone();
        T::spawn(data, api)
    }
}
#[derive(Clone)]
pub struct WindowApi {
    // todo: this actually has to be cloned, not refer to the same value!!!!
    api: Arc<Mutex<UiApi>>
}

impl WindowApi {
    pub fn new(api: UiApi) -> Self {
        WindowApi {
            api: Arc::new(Mutex::new(api))
        }
    }
}