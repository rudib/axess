use std::cell::RefCell;
use fractal_backend::{UiPayload, UiApi};
use futures::executor::block_on;

pub trait FractalWindow {
    fn set_window_api(&mut self, api: WindowApi);
    fn get_window_api(&self) -> &Option<WindowApi>;

    fn send(&self, payload: UiPayload) {
        let window_api = self.get_window_api_initialized();
        let api = window_api.api.borrow_mut();
        block_on(api.channel.send(&payload)).unwrap();
    }

    fn recv(&self) -> Option<UiPayload> {
        let window_api = self.get_window_api_initialized();
        block_on(window_api.api.borrow_mut().channel.recv())
    }

    fn get_window_api_initialized(&self) -> &WindowApi {
        self.get_window_api().as_ref().expect("should be initialized")
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