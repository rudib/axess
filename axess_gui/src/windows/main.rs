use nwd::NwgUi;
use fractal_protocol::{effect::EffectId, messages::effects::EffectStatus};
use native_windows_gui::{Tab, TabsContainer};
use log::trace;

use std::{cell::RefCell};

use axess_core::{payload::{PayloadConnection, UiPayload, DeviceState, PresetAndScene}, payload};
use super::{common::{FractalWindow, WindowApi}, connect::ConnectWindow};
use crate::{device_state::FrontendDeviceState, windows::main::main_window_ui::MainWindowUi};

// Stretch style
use nwg::stretch::{geometry::{Size, Rect}, style::{Dimension as D, FlexDirection, AlignSelf}};

const SCENE_WIDTH: D = D::Points(300.0);

const STATUS_SIZE: Size<D> = Size { width: D::Auto, height: D::Points(50.0) };
const STATUS_SCENES_SIZE: Size<D> = Size { width: SCENE_WIDTH, height: D::Auto };

const L_SIZE: Size<D> = Size { width: D::Auto, height: D::Points(40.0) };

const PRESETS_SIZE: Size<D> = Size { width: D::Auto, height: D::Auto };
const SCENES_SIZE: Size<D> = Size { width: SCENE_WIDTH, height: D::Auto };

const BLOCKS_SIZE: Size<D> = Size { width: D::Auto, height: D::Points(120.0) };

const BLOCKS_ENABLED_BUTTON_SIZE: Size<D> = Size { width: SCENE_WIDTH, height: D::Auto };

const ZERO: D = D::Points(0.0);
const NULL_PADDING: Rect<D> = Rect { top: ZERO, bottom: ZERO, start: ZERO, end: ZERO };

const MAIN_PADDING: Rect<D> = Rect { top: D::Points(5.0), bottom: D::Points(45.0), start: D::Points(5.0), end: D::Points(5.0) };

const BLOCKS_PADDING: Rect<D> = Rect { top: D::Points(5.0), bottom: D::Points(5.0), start: D::Points(10.0), end: D::Points(10.0) };

#[derive(NwgUi, Default)]
pub struct MainWindow {
    #[nwg_control(title: "Axess Fractal Audio Editor", size: (1000, 563), flags: "MAIN_WINDOW|VISIBLE")]
    #[nwg_events( OnInit: [MainWindow::init], OnWindowClose: [MainWindow::on_exit], OnKeyPress: [MainWindow::on_key_press(SELF, EVT_DATA)] )]
    window: nwg::Window,


    #[nwg_control(text: "&Device")]
    menu_device: nwg::Menu,

    #[nwg_control(text: "Connect", parent: menu_device)]
    #[nwg_events( OnMenuItemSelected: [MainWindow::connect] )]
    menu_device_connect: nwg::MenuItem,

    #[nwg_control(text: "Disconnect", parent: menu_device, disabled: true)]
    #[nwg_events( OnMenuItemSelected: [MainWindow::disconnect] )]
    menu_device_disconnect: nwg::MenuItem,

    #[nwg_control(parent: menu_device)]
    menu_device_sep: nwg::MenuSeparator,

    #[nwg_control(text: "Exit", parent: menu_device)]
    #[nwg_events( OnMenuItemSelected: [MainWindow::on_exit] )]
    menu_device_exit: nwg::MenuItem,

    #[nwg_control(text: "&Help")]
    menu_help: nwg::Menu,

    #[nwg_control(text: "Keyboard shortcuts", parent: menu_help)]
    menu_help_shortcuts: nwg::MenuItem,

    #[nwg_control(text: "About", parent: menu_help)]
    menu_help_about: nwg::MenuItem,

    #[nwg_layout(parent: window, flex_direction: FlexDirection::Column, padding: MAIN_PADDING)]
    layout: nwg::FlexboxLayout,


    #[nwg_control(flags: "VISIBLE")]
    #[nwg_layout_item(layout: layout, size: STATUS_SIZE)]
    frame_status: nwg::Frame,

    #[nwg_layout(parent: frame_status, flex_direction: FlexDirection::Row)]
    frame_status_layout: nwg::FlexboxLayout,

    #[nwg_control(parent: frame_status)]
    #[nwg_layout_item(layout: frame_status_layout, flex_grow: 2.0)]
    status_preset: nwg::Label,

    #[nwg_control(parent: frame_status)]
    #[nwg_layout_item(layout: frame_status_layout, size: STATUS_SCENES_SIZE)]
    status_scene: nwg::Label,



    #[nwg_control(flags: "VISIBLE")]
    #[nwg_layout_item(layout: layout, flex_grow: 2.0)]
    frame_presets_and_scenes: nwg::Frame,

    #[nwg_layout(parent: frame_presets_and_scenes, flex_direction: FlexDirection::Row)]
    frame_presets_ands_scenes_layout: nwg::FlexboxLayout,


    #[nwg_control(parent: frame_presets_and_scenes, flags: "VISIBLE")]
    #[nwg_layout_item(layout: frame_presets_ands_scenes_layout, size: PRESETS_SIZE, flex_grow: 2.0)]
    frame_presets: nwg::Frame,

    #[nwg_layout(parent: frame_presets, flex_direction: FlexDirection::Column, padding: NULL_PADDING)]
    frame_presets_layout: nwg::FlexboxLayout,
    
    
    #[nwg_control(parent: frame_presets, text: "Presets")]
    #[nwg_layout_item(layout: frame_presets_layout, size: L_SIZE)]
    #[nwg_layout_item(layout: frame_presets_layout)]
    presets_label_presets: nwg::Label,
    
    #[nwg_control(parent: frame_presets, list_style: nwg::ListViewStyle::Simple)]
    #[nwg_layout_item(layout: frame_presets_layout, flex_grow: 2.0)]
    #[nwg_events(OnListViewItemActivated: [MainWindow::presets_list_item_activated(SELF, EVT_DATA)], OnKeyPress: [MainWindow::presets_list_keypress(SELF, EVT_DATA)])]
    presets_list: nwg::ListView,

    
    #[nwg_control(parent: frame_presets_and_scenes, flags: "VISIBLE")]
    #[nwg_layout_item(layout: frame_presets_ands_scenes_layout, size: SCENES_SIZE)]
    frame_scenes: nwg::Frame,

    #[nwg_layout(parent: frame_scenes, flex_direction: FlexDirection::Column, padding: NULL_PADDING)]
    frame_scenes_layout: nwg::FlexboxLayout,
    

    #[nwg_control(parent: frame_scenes, text: "Scenes")]
    #[nwg_layout_item(layout: frame_scenes_layout, size: L_SIZE)]
    #[nwg_layout_item(layout: frame_scenes_layout)]
    presets_label_scenes: nwg::Label,


    #[nwg_control(parent: frame_scenes, list_style: nwg::ListViewStyle::Simple)]
    #[nwg_layout_item(layout: frame_scenes_layout, flex_grow: 2.0)]
    #[nwg_events(OnListViewItemActivated: [MainWindow::scenes_list_item_activated(SELF, EVT_DATA)], OnKeyPress: [MainWindow::scenes_list_keypress(SELF, EVT_DATA)])]
    scenes_list: nwg::ListView,



    #[nwg_control(flags: "VISIBLE")]
    #[nwg_layout_item(layout: layout, size: BLOCKS_SIZE)]
    frame_blocks: nwg::Frame,

    #[nwg_layout(parent: frame_blocks, flex_direction: FlexDirection::Column, padding: NULL_PADDING)]
    frame_blocks_layout: nwg::FlexboxLayout,

    #[nwg_control(parent: frame_blocks)]
    #[nwg_layout_item(layout: frame_blocks_layout, margin: BLOCKS_PADDING)]
    #[nwg_events( OnComboxBoxSelection: [MainWindow::blocks_on_select] )]
    blocks_list: nwg::ComboBox<String>,

    #[nwg_control(parent: frame_blocks, flags: "VISIBLE")]
    #[nwg_layout_item(layout: frame_blocks_layout)]
    frame_blocks_details: nwg::Frame,

    #[nwg_layout(parent: frame_blocks_details, flex_direction: FlexDirection::Row)]
    frame_blocks_details_layout: nwg::FlexboxLayout,

    #[nwg_control(parent: frame_blocks_details)]
    #[nwg_layout_item(layout: frame_blocks_details_layout, flex_grow: 2.0)]
    blocks_name: nwg::Label,

    #[nwg_control(parent: frame_blocks_details)]
    #[nwg_layout_item(layout: frame_blocks_details_layout, size: BLOCKS_ENABLED_BUTTON_SIZE)]
    #[nwg_events(OnButtonClick: [MainWindow::effect_bypass_toggle])]
    blocks_bypass_toggle: nwg::Button,

    
    #[nwg_control(text: "", parent: window)]
    status_bar: nwg::StatusBar,


    #[nwg_control]
    #[nwg_events( OnNotice: [MainWindow::backend_response] )]
    backend_response_notifier: nwg::Notice,



    pub ui_api: Option<WindowApi>,

    pub device_state: RefCell<FrontendDeviceState>,
    pub is_connected: RefCell<bool>,
    axess_status_bar: RefCell<AxessStatusBar>
}

impl FractalWindow for MainWindow {
    type Data = ();
    type WindowUi = MainWindowUi;
    type Window = MainWindow;

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

impl MainWindow {
    fn main_controls_when_connected(&self, visibility: bool) {
        self.frame_presets_and_scenes.set_visible(visibility);
        self.frame_blocks.set_visible(visibility);
        self.frame_status.set_visible(visibility);
    }

    fn init(&self) {
        self.main_controls_when_connected(false);
        self.axess_status_bar.borrow_mut().op(&self.status_bar).push_message(AxessStatusBarMessageKind::Default, "Not connected.".into());
        self.send(UiPayload::Connection(PayloadConnection::TryToAutoConnect));        
    }

    fn connect(&self) {
        // todo: check if we can auto connect, without a window
        //self.send(UiPayload::Connection(PayloadConnection::TryToAutoConnect));
        self.spawn_child::<ConnectWindow>(());
    }

    fn disconnect(&self) {
        self.send(UiPayload::Connection(PayloadConnection::Disconnect));
    }

    fn backend_response(&self) {
        match self.recv() {
            Some(UiPayload::Connection(PayloadConnection::AutoConnectDeviceNotFound)) => {
                // start the connect window
                //self.spawn_child::<ConnectWindow>(());
            },
            Some(UiPayload::Connection(PayloadConnection::Connected { ref device })) => {
                self.main_controls_when_connected(true);
                self.axess_status_bar.borrow_mut().op(&self.status_bar).push_message(AxessStatusBarMessageKind::Connected, format!("Connected to {}.", device));
                self.menu_device_connect.set_enabled(false);
                self.menu_device_disconnect.set_enabled(true);   
                *self.is_connected.borrow_mut() = true;

                // request the presets
                self.send(UiPayload::RequestScenes);
                self.send(UiPayload::RequestAllPresets);
            },
            Some(UiPayload::Connection(PayloadConnection::Disconnect)) => {
                self.main_controls_when_connected(false);
                self.axess_status_bar.borrow_mut().op(&self.status_bar).pop_message(AxessStatusBarMessageKind::Connected);
                self.menu_device_connect.set_enabled(true);
                self.menu_device_disconnect.set_enabled(false);
                *self.is_connected.borrow_mut() = false;
            },
            Some(UiPayload::DeviceState(DeviceState::PresetAndScene(ref p))) => {
                self.status_preset.set_text(&format!("{:0>3} {}", p.preset, p.preset_name));
                self.status_scene.set_text(&format!("Scene {} {}", p.scene + 1, p.scene_name));

                // todo: select in the items

                let ref mut device_state = self.device_state.borrow_mut();
                device_state.current_preset_and_scene = Some(p.clone());
            },
            Some(UiPayload::Presets(presets)) => {
                self.presets_list.clear();
                for p in &presets {
                    self.presets_list.insert_item(format!("{:0>3} {}", p.number, p.name));
                }
                self.presets_list.set_visible(true);
                self.presets_list.set_focus();

                let mut state = self.device_state.borrow_mut();
                state.presets = presets;
                if let Some(ref current_preset) = state.current_preset_and_scene {                
                    self.presets_list.select_item(current_preset.preset as usize, true);
                }
            },
            Some(UiPayload::Scenes(scenes)) => {
                self.scenes_list.clear();
                for s in &scenes {
                    self.scenes_list.insert_item(format!("Scene {} {}", s.number + 1, s.name));
                }
                self.scenes_list.set_visible(true);

                let mut state = self.device_state.borrow_mut();
                state.current_presets_scenes = scenes;
                if let Some(ref current_preset) = state.current_preset_and_scene {
                    self.scenes_list.select_item(current_preset.scene as usize, true);
                }
            },
            Some(UiPayload::CurrentBlocks(blocks)) => {

            },
            Some(UiPayload::EffectStatus(effects)) => {
                let l: Vec<_> = effects.0.iter()
                    .filter(|x| {
                        x.effect_id != EffectId::ID_EFFECTS_END && x.effect_id != EffectId::ID_PRESET_FC
                    })
                    .map(|x| {
                        if let Some(display_name) = x.effect_id.get_display_name() {
                            display_name.into()
                        } else {
                            format!("{:?}", x.effect_id)
                        }
                    })
                    .collect();
                let len = l.len();
                
                {
                    let mut state = self.device_state.borrow_mut();
                    state.current_effects = Some(effects);
                }
                
                let diff = !self.blocks_list.collection().eq(&l);
                self.blocks_list.set_collection(l);                
                if diff && len > 0 {
                    self.blocks_list.set_selection(Some(0));
                }
                self.blocks_on_select();
            },

            Some(UiPayload::EffectBypassStatus(effect_bypass_status)) => {                
                {
                    let mut state = self.device_state.borrow_mut();
                    
                    if let Some(ref mut effects) = state.current_effects {
                        for mut ef in &mut effects.0 {
                            if ef.effect_id == effect_bypass_status.effect_id {
                                ef.is_bypassed = effect_bypass_status.is_bypassed;
                            }
                        }
                    }
                }

                self.blocks_on_select();
            }

            Some(UiPayload::ProgressReport { i, total }) => {
                let mut s = self.axess_status_bar.borrow_mut();
                let mut op = s.op(&self.status_bar);
                if i+1 == total {
                    op.pop_message(AxessStatusBarMessageKind::Progress);
                } else {
                    op.push_message(AxessStatusBarMessageKind::Progress, format!("Loading preset name {}/{} ...", (i+1), total));
                }
            }

            Some(_) => {}
            None => {}
        }
    }

    fn preset_delta(&self, delta: i16) {
        let device_state = self.device_state.borrow();
        if let Some(ref current_preset) = device_state.current_preset_and_scene {

            let mut p = current_preset.preset as i16 + delta;
            if p < 0 { p = 511; }
            if p > 511 { p = 0; }

            self.send(UiPayload::DeviceState(DeviceState::SetPreset { preset: p as u16 }));
        }
    }

    fn previous_preset(&self) {
        self.preset_delta(-1);
    }

    fn next_preset(&self) {
        self.preset_delta(1);
    }

    fn scene_delta(&self, delta: i8) {
        let device_state = self.device_state.borrow();
        if let Some(ref current_preset) = device_state.current_preset_and_scene {

            let mut s = current_preset.scene as i8 + delta;
            if s < 0 { s = 7; }
            if s > 7 { s = 0; }

            self.send(UiPayload::DeviceState(DeviceState::SetScene { scene: s as u8 }));
        }
    }

    fn previous_scene(&self) {
        self.scene_delta(-1);
    }

    fn next_scene(&self) {
        self.scene_delta(1);
    }

    fn on_key_press(&self, data: &nwg::EventData) {
        if *self.is_connected.borrow() == false { return; }

        if let nwg::EventData::OnKey(key) = data {
            if *key == 'W' as u32 {
                self.previous_scene();
            } else if *key == 'S' as u32 {
                self.next_scene();
            } else if *key == 'D' as u32 {
                self.next_preset();
            } else if *key == 'A' as u32 {
                self.previous_preset();
            }
        }
    }

    fn presets_list_item_activated(&self, _data: &nwg::EventData) {
        self.preset_selected();
    }

    fn presets_list_keypress(&self, data: &nwg::EventData) {
        if let nwg::EventData::OnKey(key) = data {
            if *key == ' ' as u32 {
                self.preset_selected();
            }
        }
    }

    fn preset_selected(&self) {
        if let Some(idx) = self.presets_list.selected_item() {
            trace!("Selecting preset {}", idx);
            self.send(UiPayload::DeviceState(payload::DeviceState::SetPreset {preset: idx as u16 }));
        }
    }

    fn scenes_list_item_activated(&self, _data: &nwg::EventData) {
        self.scene_selected();
    }

    fn scenes_list_keypress(&self, data: &nwg::EventData) {
        if let nwg::EventData::OnKey(key) = data {
            if *key == ' ' as u32 {
                self.scene_selected();
            }
        }
    }

    fn scene_selected(&self) {
        if let Some(idx) = self.scenes_list.selected_item() {
            trace!("Selecting scene {}", idx);
            self.send(UiPayload::DeviceState(payload::DeviceState::SetScene {scene: idx as u8 }));
        }
    }

    fn get_current_selected_effect(&self) -> Option<EffectStatus> {
        if let Some(idx) = self.blocks_list.selection() {
            let state = self.device_state.borrow();
            if let Some(ref effects) = state.current_effects {
                return effects.0.get(idx).cloned();
            }
        }

        None
    }

    fn effect_bypass_toggle(&self) {
        if let Some(effect) = self.get_current_selected_effect() {
            let new_status = !effect.is_bypassed;
            trace!("Setting effect {:?} to status {}", effect.effect_id, if new_status { "DISABLED" } else { "ENABLED" });

            self.send(UiPayload::SetEffectBypass { effect: effect.effect_id, is_bypassed: new_status });
        }
    }

    fn blocks_on_select(&self) {
        if let Some(effect) = self.get_current_selected_effect() {
            let effect_id_name = if let Some(display_name) = effect.effect_id.get_display_name() {
                display_name.into()
            } else {
                format!("{:?}", effect.effect_id)
            };

            self.blocks_name.set_text(&format!("{}, channel {:?}", effect_id_name, effect.channel));
            
            let button_label = if effect.is_bypassed {
                "DISABLED"
            } else {
                "ENABLED"
            };
            self.blocks_bypass_toggle.set_text(button_label);
        }
    }
}


#[derive(Default, Clone)]
struct AxessStatusBar {
    messages: Vec<(AxessStatusBarMessageKind, String)>
}

impl AxessStatusBar {
    fn op<'a, 'b>(&'a mut self, status_bar: &'b nwg::StatusBar) -> AxessStatusBarOperation<'a, 'b> {
        AxessStatusBarOperation {
            s: self,
            status_bar
        }
    }
}

struct AxessStatusBarOperation<'a, 'b> {
    s: &'a mut AxessStatusBar,
    status_bar: &'b nwg::StatusBar
}

impl<'a, 'b> AxessStatusBarOperation<'a, 'b> {
    fn push_message(&mut self, message_kind: AxessStatusBarMessageKind, message: String) {
        let existing = self.s.messages.iter_mut().find(|p| p.0 == message_kind);
        if let Some(existing) = existing {
            existing.1 = message;
        } else {
            self.s.messages.push((message_kind, message));
        }
    }

    fn pop_message(&mut self, message_kind: AxessStatusBarMessageKind) {
        self.s.messages.retain(|m| m.0 != message_kind);
    }
}

impl<'a, 'b> Drop for AxessStatusBarOperation<'a, 'b> {
    fn drop(&mut self) {
        if let Some(msg) = self.s.messages.last() {
            self.status_bar.set_text(0, &msg.1);
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum AxessStatusBarMessageKind {
    Default,
    Connected,
    Progress
}