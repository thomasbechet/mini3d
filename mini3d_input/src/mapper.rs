use std::fmt::Debug;
use std::hash::Hash;
use std::{collections::HashMap, fs::File};

use mini3d_core::input::component::{
    self, InputActionHandle, InputAxis, InputAxisHandle, InputAxisRange,
};
use mini3d_core::input::provider::{InputProviderError, InputProviderHandle};
use mini3d_core::math::fixed::I32F16;
use mini3d_core::utils::uid::ToUID;
use mini3d_core::{
    input::{
        event::{InputActionEvent, InputAxisEvent, InputEvent, InputTextEvent},
        provider::InputProvider,
    },
    utils::uid::UID,
};
use serde::{Deserialize, Deserializer, Serialize};

pub trait InputMapperButton:
    Sized + Copy + Clone + Hash + Eq + Debug + Serialize + for<'de> Deserialize<'de>
{
}
pub trait InputMapperAxis:
    Sized + Copy + Clone + Hash + Eq + Debug + Serialize + for<'de> Deserialize<'de>
{
}

struct ButtonToAction {
    handle: InputActionHandle,
    was_pressed: bool,
}
struct ButtonToAxis {
    handle: InputAxisHandle,
    value: f32,
}
struct AxisToAxis {
    handle: InputAxisHandle,
    scale: f32,
    deadzone: f32,
}

#[derive(Default, Clone, Serialize)]
pub struct MapActionInput<T: InputMapperButton> {
    name: String,
    button: Option<T>,
    #[serde(skip)]
    handle: Option<InputActionHandle>,
}

impl<'de, T: InputMapperButton> serde::Deserialize<'de> for MapActionInput<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let (name, button) = <(String, Option<T>)>::deserialize(d)?;
        Ok(Self {
            name,
            button,
            handle: None,
        })
    }
}

#[derive(Default, Clone, Serialize)]
pub struct MapAxisInput<B: InputMapperButton, A: InputMapperAxis> {
    pub name: String,
    pub button: Option<(B, f32)>,    // Button, Scale
    pub axis: Option<(A, f32, f32)>, // Axis, Scale, Deadzone
    #[serde(skip)]
    pub handle: Option<(InputAxisHandle, InputAxisRange)>,
}

impl<'de, B: InputMapperButton, A: InputMapperAxis> serde::Deserialize<'de> for MapAxisInput<B, A> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let (name, button, axis) =
            <(String, Option<(B, f32)>, Option<(A, f32, f32)>)>::deserialize(d)?;
        Ok(Self {
            name,
            handle: None,
            button,
            axis,
        })
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InstanceId(u32);

#[derive(Default, Clone, Serialize)]
pub struct InputProfile<B: InputMapperButton, A: InputMapperAxis> {
    pub name: String,
    pub active: bool,
    pub instance: Option<InstanceId>,
    pub actions: Vec<MapActionInput<B>>,
    pub axis: Vec<MapAxisInput<B, A>>,
}

impl<'de, B: InputMapperButton, A: InputMapperAxis> serde::Deserialize<'de> for InputProfile<B, A> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let (name, active, actions, axis) = <(
            String,
            bool,
            Vec<MapActionInput<B>>,
            Vec<MapAxisInput<B, A>>,
        )>::deserialize(d)?;
        Ok(Self {
            name,
            active,
            actions,
            axis,
        })
    }
}

struct InstanceProvider {
    id: u32,
    events: Vec<InputEvent>,
}

#[derive(Default)]
pub struct InputMapper<B: InputMapperButton, A: InputMapperAxis> {
    profiles: HashMap<UID, InputProfile<B, A>>,
    default_profile: UID,
    instances: HashMap<InstanceId, Instance>,

    button_to_action: HashMap<B, Vec<ButtonToAction>>,
    button_to_axis: HashMap<B, Vec<ButtonToAxis>>,
    axis_to_axis: HashMap<A, Vec<AxisToAxis>>,
}

impl<B: InputMapperButton, A: InputMapperAxis> InputMapper<B, A> {
    pub fn new() -> Self {
        InputMapper {
            default_profile: UID::from("Default"),
            profiles: Default::default(),
            events: Default::default(),
            button_to_action: Default::default(),
            button_to_axis: Default::default(),
            axis_to_axis: Default::default(),
        }
    }

    pub fn new_profile(&mut self) -> UID {
        let mut next_index = self.profiles.len() + 1;
        let mut name = format!("Profile {}", next_index);
        let uid = UID::from(&name);
        while self.profiles.iter().any(|(_, p)| p.name == name) {
            next_index += 1;
            name = format!("Profile {}", next_index);
        }
        self.profiles.insert(
            uid,
            InputProfile {
                name,
                active: true,
                actions: Default::default(),
                axis: Default::default(),
            },
        );
        self.rebuild_cache();
        uid
    }

    pub fn remove_profile(&mut self, profile: UID) {
        self.profiles.remove(&profile);
        self.rebuild_cache();
    }

    pub fn default_profile(&self) -> UID {
        self.default_profile
    }

    pub fn iter_profiles(&self) -> impl Iterator<Item = (UID, &InputProfile<B, A>)> {
        self.profiles.iter().map(|(k, v)| (*k, v))
    }

    pub fn iter_actions(&self, profile: UID) -> impl Iterator<Item = &MapActionInput<B>> {
        self.profiles
            .get(&profile)
            .map(|p| p.actions.iter())
            .unwrap_or_default()
    }

    pub fn iter_axis(&self, profile: UID) -> impl Iterator<Item = &MapAxisInput<B, A>> {
        self.profiles
            .get(&profile)
            .map(|p| p.axis.iter())
            .unwrap_or_default()
    }

    pub fn bind_button_to_action(&mut self, profile: UID, entry: UID, button: Option<B>) -> bool {
        if let Some(profile) = self.profiles.get_mut(&profile) {
            if let Some(action) = profile
                .actions
                .iter_mut()
                .find(|a| a.name.to_uid() == entry)
            {
                action.button = button;
                self.rebuild_cache();
                return true;
            }
        }
        false
    }

    pub fn bind_axis_to_axis(
        &mut self,
        profile: UID,
        entry: UID,
        value: Option<(A, f32, f32)>,
    ) -> bool {
        if let Some(profile) = self.profiles.get_mut(&profile) {
            if let Some(ax) = profile.axis.iter_mut().find(|a| a.name.to_uid() == entry) {
                ax.axis = value;
                self.rebuild_cache();
                return true;
            }
        }
        false
    }

    pub fn bind_button_to_axis(
        &mut self,
        profile: UID,
        entry: UID,
        value: Option<(B, f32)>,
    ) -> bool {
        if let Some(profile) = self.profiles.get_mut(&profile) {
            if let Some(axis) = profile.axis.iter_mut().find(|a| a.name.to_uid() == entry) {
                axis.button = value;
                self.rebuild_cache();
                return true;
            }
        }
        false
    }

    pub fn duplicate_profile(&mut self, from: UID) -> UID {
        if let Some(from) = self.profiles.get(&from) {
            let mut name = format!("{} Copy", from.name);
            let mut next_index = 1;
            while self.profiles.iter().any(|(_, p)| p.name == name) {
                next_index += 1;
                name = format!("{} Copy {}", from.name, next_index);
            }
            let uid = UID::from(&name);
            let profile = InputProfile {
                name,
                active: true,
                actions: from.actions.clone(),
                axis: from.axis.clone(),
            };
            self.profiles.insert(uid, profile);
            self.rebuild_cache();
            uid
        } else {
            UID::null()
        }
    }

    pub fn set_default_profile(&mut self, profile: UID) {
        self.default_profile = profile;
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        std::fs::create_dir_all("config").unwrap();
        let file = File::create("config/profiles.json")?;
        let profiles = self.profiles.values().collect::<Vec<&_>>();
        serde_json::to_writer_pretty(&file, &profiles)?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), std::io::Error> {
        let file = File::open("config/profiles.json")?;
        let mut profiles: Vec<InputProfile<B, A>> = serde_json::from_reader(&file).unwrap();
        for profile in profiles.drain(..) {
            if let Some((_, current)) = self
                .profiles
                .iter_mut()
                .find(|(_, p)| p.name == profile.name)
            {
                *current = profile;
            } else {
                let uid = UID::from(&profile.name);
                self.profiles.insert(uid, profile);
            }
        }
        Ok(())
    }

    pub fn rebuild_cache(&mut self) {
        // Clear caches
        self.button_to_action.clear();
        self.button_to_axis.clear();
        self.axis_to_axis.clear();

        // Update caches
        for profile in self.profiles.values() {
            if profile.active {
                for action in &profile.actions {
                    if let Some(button) = &action.button {
                        if let Some(action_handle) = action.handle {
                            self.button_to_action.entry(*button).or_default().push(
                                ButtonToAction {
                                    handle: action_handle,
                                    was_pressed: false,
                                },
                            );
                        }
                    }
                }
                for axis in &profile.axis {
                    if let Some((b, value)) = &axis.button {
                        if let Some((axis_handle, _)) = axis.handle {
                            self.button_to_axis
                                .entry(*b)
                                .or_default()
                                .push(ButtonToAxis {
                                    handle: axis_handle,
                                    value: *value,
                                });
                        }
                    }
                    if let Some((a, scale, deadzone)) = &axis.axis {
                        if let Some((axis_handle, _)) = axis.handle {
                            self.axis_to_axis.entry(*a).or_default().push(AxisToAxis {
                                handle: axis_handle,
                                scale: *scale,
                                deadzone: *deadzone,
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn dispatch_text(&mut self, value: String) {
        self.events.push(InputEvent::Text(InputTextEvent {
            handle: Default::default(),
            value,
        }));
    }

    pub fn dispatch_button(&mut self, button: B, pressed: bool) {
        if let Some(actions) = self.button_to_action.get_mut(&button) {
            for action in actions {
                // Prevent repeating events
                if action.was_pressed != pressed {
                    self.events.push(InputEvent::Action(InputActionEvent {
                        handle: action.handle,
                        pressed,
                    }));
                    action.was_pressed = pressed;
                }
            }
        }
        if let Some(axis) = self.button_to_axis.get(&button) {
            for ax in axis {
                let value = if pressed { ax.value } else { 0.0 };
                self.events.push(InputEvent::Axis(InputAxisEvent {
                    handle: ax.handle,
                    value: I32F16::from_f32(value),
                }));
            }
        }
    }

    pub fn dispatch_axis(&mut self, axis: A, value: f32) {
        if let Some(axis) = self.axis_to_axis.get(&axis) {
            for ax in axis {
                if value.abs() >= ax.deadzone {
                    self.events.push(InputEvent::Axis(InputAxisEvent {
                        handle: ax.handle,
                        value: I32F16::from_f32(value * ax.scale),
                    }));
                }
            }
        }
    }

    // pub fn dispatch_controller_axis(
    //     &mut self,
    //     id: T::ControllerId,
    //     controller_axis: T::ControllerAxis,
    //     value: f32,
    // ) {
    //     // Compute value with deadzone
    //     let value = if f32::abs(value) <= 0.15 { 0.0 } else { value };

    //     if let Some(axis) = &self.controllers_axis_to_axis.get(&id) {
    //         if let Some(ax) = axis.get(&controller_axis) {
    //             for a in ax {
    //                 self.events.push(InputEvent::Axis(InputAxisEvent {
    //                     handle: a.handle,
    //                     value: I32F16::from_f32(value * a.scale),
    //                 }));
    //             }
    //         }
    //     }
    // }
}

impl<B: InputMapperButton, A: InputMapperAxis> InputProvider for InputMapper<B, A> {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_event(&mut self) -> Option<InputEvent> {
        self.events.pop()
    }

    fn add_action(
        &mut self,
        name: &str,
        action: &component::InputAction,
        handle: InputActionHandle,
    ) -> Result<InputProviderHandle, InputProviderError> {
        for profile in self.profiles.values_mut() {
            if let Some(a) = profile.actions.iter_mut().find(|a| a.name == name) {
                a.handle = Some(handle);
            }
            profile.actions.push(MapActionInput {
                name: name.to_string(),
                handle: Some(handle),
                button: None,
            });
        }
        self.rebuild_cache();
        Ok(Default::default())
    }

    fn remove_action(&mut self, handle: InputProviderHandle) -> Result<(), InputProviderError> {
        Ok(())
    }

    fn add_axis(
        &mut self,
        name: &str,
        axis: &InputAxis,
        handle: InputAxisHandle,
    ) -> Result<InputProviderHandle, InputProviderError> {
        for profile in self.profiles.values_mut() {
            if let Some(a) = profile.axis.iter_mut().find(|a| a.name == name) {
                a.handle = Some((handle, axis.range));
            }
            profile.axis.push(MapAxisInput {
                name: name.to_string(),
                handle: Some((handle, axis.range)),
                button: None,
                axis: None,
            });
        }
        self.rebuild_cache();
        Ok(Default::default())
    }

    fn remove_axis(&mut self, handle: InputProviderHandle) -> Result<(), InputProviderError> {
        Ok(())
    }
}
