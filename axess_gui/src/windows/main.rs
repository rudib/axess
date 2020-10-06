use nwd::NwgUi;
use fractal_protocol::{effect::EffectId, messages::effects::EffectStatus};
use native_windows_gui::{TabsContainer, Tab};
use log::trace;

use std::{cell::RefCell};

use axess_core::{payload::{PayloadConnection, UiPayload, DeviceState, PresetAndScene}, payload};
use super::{common::{FractalWindow, WindowApi}, connect::ConnectWindow};
use crate::{device_state::FrontendDeviceState, windows::main::main_window_ui::MainWindowUi};

const NOT_CONNECTED: &'static str = "Not connected.";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tabs {
    Main,
    Presets,
    Blocks
}

#[derive(NwgUi, Default)]
pub struct MainWindow {
    #[nwg_control(title: "Axess Fractal Audio Editor", size: (800, 600), flags: "MAIN_WINDOW|VISIBLE")]
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


    #[nwg_control(text: "&View")]
    menu_view: nwg::Menu,
    #[nwg_control(text: "&Main", parent: menu_view)]
    #[nwg_events( OnMenuItemSelected: [MainWindow::set_selected_tab_main(SELF)] )]
    menu_view_main: nwg::MenuItem,
    #[nwg_control(text: "&Presets", parent: menu_view)]
    #[nwg_events( OnMenuItemSelected: [MainWindow::set_selected_tab_presets(SELF)] )]
    menu_view_presets: nwg::MenuItem,
    #[nwg_control(text: "&Blocks", parent: menu_view)]
    #[nwg_events( OnMenuItemSelected: [MainWindow::set_selected_tab_blocks(SELF)] )]
    menu_view_blocks: nwg::MenuItem,


    /*
    #[nwg_control(text: "&Help")]
    menu_help: nwg::Menu,

    #[nwg_control(text: "About", parent: menu_help)]
    menu_help_about: nwg::MenuItem,
    */

    #[nwg_control(text: NOT_CONNECTED, parent: window)]
    status_bar: nwg::StatusBar,

    #[nwg_layout(parent: window)]
    window_layout: nwg::GridLayout,

    #[nwg_control(parent: window)]
    #[nwg_layout_item(layout: window_layout, row: 0, col: 0)]
    #[nwg_events(TabsContainerChanged: [MainWindow::on_tab_changed])]
    tabs_holder: TabsContainer,
    #[nwg_control(parent: tabs_holder, text: "&Main")]
    tab_main: Tab,
    #[nwg_control(parent: tabs_holder, text: "&Presets")]
    tab_presets: Tab,
    #[nwg_control(parent: tabs_holder, text: "&Blocks")]
    tab_blocks: Tab,



    #[nwg_layout(parent: tab_main, spacing: 1)]
    main_grid: nwg::GridLayout,

    #[nwg_control(parent: tab_main)]
    #[nwg_layout_item(layout: main_grid, row: 0, col: 0)]
    main_preset_number: nwg::Label,

    #[nwg_control(parent: tab_main)]
    #[nwg_layout_item(layout: main_grid, row: 0, col: 1)]
    main_preset_name: nwg::Label,

    #[nwg_control(text: "Previous Preset", parent: tab_main)]
    #[nwg_layout_item(layout: main_grid, row: 1, col: 0)]
    #[nwg_events(OnButtonClick: [MainWindow::previous_preset])]
    main_preset_minus: nwg::Button,
    
    #[nwg_control(text: "Next Preset", parent: tab_main)]
    #[nwg_layout_item(layout: main_grid, row: 1, col: 1)]
    #[nwg_events(OnButtonClick: [MainWindow::next_preset])]
    main_preset_plus: nwg::Button,

    #[nwg_control(parent: tab_main)]
    #[nwg_layout_item(layout: main_grid, row: 2, col: 0)]
    main_scene_number: nwg::Label,

    #[nwg_control(parent: tab_main)]
    #[nwg_layout_item(layout: main_grid, row: 2, col: 1)]
    main_scene_name: nwg::Label,

    #[nwg_control(text: "Previous Scene", parent: tab_main)]
    #[nwg_layout_item(layout: main_grid, row: 3, col: 0)]
    #[nwg_events(OnButtonClick: [MainWindow::previous_scene])]
    main_scene_minus: nwg::Button,
    
    #[nwg_control(text: "Next Scene", parent: tab_main)]
    #[nwg_layout_item(layout: main_grid, row: 3, col: 1)]
    #[nwg_events(OnButtonClick: [MainWindow::next_scene])]
    main_scene_plus: nwg::Button,


    
    #[nwg_layout(parent: tab_presets)]
    presets_grid: nwg::GridLayout,

    #[nwg_control(parent: tab_presets, text: "Presets (use SPACE to select)")]
    #[nwg_layout_item(layout: presets_grid, row: 0, col: 0)]
    presets_label_presets: nwg::Label,

    #[nwg_control(parent: tab_presets, list_style: nwg::ListViewStyle::Simple)]
    #[nwg_layout_item(layout: presets_grid, row: 1, col: 0)]
    #[nwg_events(OnListViewItemActivated: [MainWindow::presets_list_item_activated(SELF, EVT_DATA)], OnKeyPress: [MainWindow::presets_list_keypress(SELF, EVT_DATA)])]
    presets_list: nwg::ListView,

    #[nwg_control(parent: tab_presets, text: "Scenes of the current preset")]
    #[nwg_layout_item(layout: presets_grid, row: 0, col: 1)]
    presets_label_scenes: nwg::Label,

    #[nwg_control(parent: tab_presets, list_style: nwg::ListViewStyle::Simple)]
    #[nwg_layout_item(layout: presets_grid, row: 1, col: 1)]
    #[nwg_events(OnListViewItemActivated: [MainWindow::scenes_list_item_activated(SELF, EVT_DATA)], OnKeyPress: [MainWindow::scenes_list_keypress(SELF, EVT_DATA)])]
    scenes_list: nwg::ListView,


    #[nwg_layout(parent: tab_blocks)]
    blocks_grid: nwg::GridLayout,

    /*
    #[nwg_control(parent: tab_blocks, text: "Blocks")]
    #[nwg_layout_item(layout: blocks_grid, row: 0, col: 0)]
    blocks_label: nwg::Label,
    */
    #[nwg_control(parent: tab_blocks)]
    #[nwg_layout_item(layout: blocks_grid, row: 0, col: 0)]
    #[nwg_events( OnComboxBoxSelection: [MainWindow::blocks_on_select] )]
    blocks_list: nwg::ComboBox<String>,


    #[nwg_control(parent: tab_blocks)]
    #[nwg_layout_item(layout: blocks_grid, row: 1, col: 0)]
    blocks_name: nwg::Label,

    #[nwg_control(parent: tab_blocks)]
    #[nwg_layout_item(layout: blocks_grid, row: 2, col: 0)]
    #[nwg_events(OnButtonClick: [MainWindow::effect_bypass_toggle])]
    blocks_bypass_toggle: nwg::Button,

    #[nwg_control(parent: tab_blocks)]
    #[nwg_layout_item(layout: blocks_grid, row: 3, col: 0)]
    blocks_channel: nwg::Label,




    #[nwg_control]
    #[nwg_events( OnNotice: [MainWindow::backend_response] )]
    backend_response_notifier: nwg::Notice,

    pub ui_api: Option<WindowApi>,

    pub device_state: RefCell<FrontendDeviceState>,
    pub is_connected: RefCell<bool>
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
    fn tabs() -> [Tabs; 3] {
        [
            Tabs::Main,
            Tabs::Presets,
            Tabs::Blocks
        ]
    }

    fn get_selected_tab(&self) -> Option<Tabs> {
        let tab_idx = self.tabs_holder.selected_tab();
        Self::tabs().get(tab_idx).cloned()
    }

    fn set_selected_tab(&self, tab: Tabs) {
        let tabs = Self::tabs();
        let idx = tabs.iter().position(|x| *x == tab);
        if let Some(idx) = idx {
            self.tabs_holder.set_selected_tab(idx);
            self.on_tab_changed();
        }
    }

    fn set_selected_tab_main(&self) { self.set_selected_tab(Tabs::Main) }
    fn set_selected_tab_presets(&self) { self.set_selected_tab(Tabs::Presets) }
    fn set_selected_tab_blocks(&self) { self.set_selected_tab(Tabs::Blocks) }

    fn main_controls_when_connected(&self, visibility: bool) {
        //self.tabs_holder.set_visible(visibility);
        //self.tab_main.set_visible(visibility);
        //self.tab_presets.set_visible(visibility);
    }

    fn init(&self) {
        self.main_controls_when_connected(false);
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
                self.status_bar.set_text(0, &format!("Connected to {}.", device));
                self.menu_device_connect.set_enabled(false);
                self.menu_device_disconnect.set_enabled(true);   
                *self.is_connected.borrow_mut() = true;
            },
            Some(UiPayload::Connection(PayloadConnection::Disconnect)) => {
                self.main_controls_when_connected(false);
                self.status_bar.set_text(0, NOT_CONNECTED);
                self.menu_device_connect.set_enabled(true);
                self.menu_device_disconnect.set_enabled(false);
                *self.is_connected.borrow_mut() = false;
            },
            Some(UiPayload::DeviceState(DeviceState::PresetAndScene(ref p))) => {
                self.main_preset_number.set_text(&format!("{:0>3}", p.preset));
                self.main_preset_name.set_text(&p.preset_name);
                self.main_scene_number.set_text(&format!("Scene {}", p.scene + 1));
                self.main_scene_name.set_text(&p.scene_name);

                self.device_state.borrow_mut().current_preset_and_scene = Some(p.clone());
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
                    self.scenes_list.insert_item(format!("Scene {} {}", s.number, s.name));
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
                        format!("{:?}", x.effect_id)
                    })
                    .collect();
                let len = l.len();
                
                {
                    let mut state = self.device_state.borrow_mut();
                    state.current_effects = Some(effects);
                }
                
                let was_empty = self.blocks_list.collection().is_empty();
                self.blocks_list.set_collection(l);                
                if was_empty && len > 0 {
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

        if self.get_selected_tab() == Some(Tabs::Main) {
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
    }

    fn on_tab_changed(&self) {
        match self.get_selected_tab() {
            Some(Tabs::Presets) => {
                self.presets_list.set_visible(false);
                self.scenes_list.set_visible(false);
                
                self.presets_list.clear();                
                self.scenes_list.clear();
                
                self.send(UiPayload::RequestAllPresets);                
                self.send(UiPayload::RequestScenes);
            }
            Some(Tabs::Blocks) => {
                self.blocks_list.collection_mut().clear();
                self.send(UiPayload::RequestEffectStatus);
            },
            _ => ()
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
        if self.tabs_holder.selected_tab() == 1 && self.presets_list.focus() {
            if let Some(idx) = self.presets_list.selected_item() {
                trace!("Selecting preset {}", idx);
                self.send(UiPayload::DeviceState(payload::DeviceState::SetPreset {preset: idx as u16 }));
            }
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
        if self.tabs_holder.selected_tab() == 1 && self.scenes_list.focus() {
            if let Some(idx) = self.scenes_list.selected_item() {
                trace!("Selecting scene {}", idx);
                self.send(UiPayload::DeviceState(payload::DeviceState::SetScene {scene: idx as u8 }));
            }
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
            self.blocks_name.set_text(&format!("{:?}", effect.effect_id));
            self.blocks_channel.set_text(&format!("Channel {:?}", effect.channel));
            let button_label = if effect.is_bypassed {
                "DISABLED"
            } else {
                "ENABLED"
            };
            self.blocks_bypass_toggle.set_text(button_label);
        }
    }
}

