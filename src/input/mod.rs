use std::{
    ops::Range,
    time::{Duration, Instant},
};

use glam::Vec3;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, Event, KeyEvent, MouseScrollDelta, WindowEvent},
    keyboard::{Key, KeyCode, PhysicalKey},
    window::WindowId,
};

use crate::input::{error::InputResult, settings::KeyMap};

mod error;
mod settings;

/// Enthält den Zustand einer Taste.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FrameState {
    NotPressed,

    JustPressed,
    Pressed,

    JustReleased,
    Released,
}

impl From<FrameState> for bool {
    fn from(state: FrameState) -> Self {
        match state {
            FrameState::JustPressed | FrameState::Pressed => true,
            FrameState::NotPressed | FrameState::Released | FrameState::JustReleased => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InputState {
    pub state: FrameState,
    pub timestamp: Instant,
}

#[allow(dead_code)]
impl InputState {
    fn frame_done(&mut self) {
        self.state = match self.state {
            FrameState::JustPressed => FrameState::Pressed,
            FrameState::JustReleased => FrameState::Released,
            _ => return,
        }
    }

    pub fn release(&mut self) {
        self.state = match self.state {
            FrameState::NotPressed | FrameState::JustPressed | FrameState::Pressed => {
                FrameState::JustReleased
            }

            FrameState::JustReleased => FrameState::Released,

            _ => return,
        };

        self.timestamp = Instant::now();
    }

    pub fn press(&mut self) {
        self.state = match self.state {
            FrameState::NotPressed | FrameState::JustReleased | FrameState::Released => {
                FrameState::JustPressed
            }
            // FrameState::JustPressed => FrameState::Pressed,
            _ => return,
        };

        self.timestamp = Instant::now()
    }

    /// Methode die zurück gibt ob eine Taste in diesem Frame gedrückt wurde.
    /// Die Methode gibt nur wahr zurück wenn die Taste im vorherigen Frame nicht gedrückt wurde.
    pub fn just_pressed(&self) -> bool {
        self.state == FrameState::JustPressed
    }

    /// Methode die zurückgibt ob die Taste gerade gedrückt ist.
    pub fn pressed(&self) -> bool {
        self.state.into()
    }

    /// Methode die zurück gibt ob eine Taste in diesem Frame losgelassen wurde.
    pub fn just_released(&self) -> bool {
        self.state == FrameState::JustReleased
    }

    /// Methode die die Zeit in Nanosekunden zurückgibt die die Taste aktuell schon gedrückt wurde.
    pub fn time_pressed(&self) -> Option<f64> {
        if self.state.into() {
            Some(self.timestamp.elapsed().as_secs_f64())
        } else {
            None
        }
    }

    pub fn timestamp(&self) -> f64 {
        if self.state.into() {
            self.timestamp.elapsed().as_secs_f64()
        } else {
            0.0
        }
    }

    /// Methode die die Zeit in Nanosekunden zurückgibt die die Taste jetzt schon losgelassen wurde.
    pub fn time_released(&self) -> Option<f64> {
        if !<FrameState as Into<bool>>::into(self.state) {
            Some(self.timestamp.elapsed().as_secs_f64())
        } else {
            None
        }
    }

    /// Methode die überprüft ob sich die Zeit die eine Taste schon gedrückt wurde in einem gegebenen Bereich befindet.
    pub fn pressed_for(&self, time: Range<f64>) -> bool {
        if <FrameState as Into<bool>>::into(self.state) {
            let time_pressed = self.timestamp.elapsed().as_secs_f64();
            time.start <= time_pressed && time_pressed <= time.end
        } else {
            false
        }
    }

    /// Methode die überprüft ob sich die Zeit die eine Taste schon nicht mehr gedrückt wurde
    /// in einem gegebenen Bereich befindet.
    pub fn released_for(&self, time: Range<f64>) -> bool {
        if let FrameState::JustReleased | FrameState::Released = self.state {
            let time_released = self.timestamp.elapsed().as_secs_f64();
            time.start <= time_released && time_released <= time.end
        } else {
            false
        }
    }
}

const VEC32_ZERO: PhysicalPosition<f32> = PhysicalPosition::new(0., 0.);
const VEC64_ZERO: PhysicalPosition<f64> = PhysicalPosition::new(0., 0.);

#[derive(Clone, Debug)]
pub struct Inputs {
    pub forward: bool,
    pub backwards: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,

    pub mouse_motion: Option<PhysicalPosition<f64>>,
    pub mouse_wheel: Option<PhysicalPosition<f32>>,

    pub pause: bool,
    pub remesh: bool,
    pub lod_up: bool,
    pub lod_down: bool,
    pub free_cam: bool,
    pub status: bool,
    pub toggle_impl: bool,

    pub space: InputState,
    pub last_space_press: Option<Instant>,
}

pub const DOUBLE_CLICK_TIMESPAN: Duration = Duration::from_millis(500);

impl Inputs {
    fn new() -> Self {
        Self {
            forward: false,
            backwards: false,
            left: false,
            right: false,
            up: false,
            down: false,

            mouse_motion: None,
            mouse_wheel: None,

            pause: false,
            remesh: false,
            lod_up: false,
            lod_down: false,
            free_cam: false,
            status: false,
            toggle_impl: false,

            space: InputState {
                state: FrameState::NotPressed,
                timestamp: Instant::now(),
            },
            last_space_press: None,
        }
    }

    pub fn input_vector(&self) -> Vec3 {
        Vec3::new(
            self.forward as u32 as f32 - self.backwards as u32 as f32,
            self.up as u32 as f32 - self.down as u32 as f32,
            self.right as u32 as f32 - self.left as u32 as f32,
        )
    }
}

pub struct InputEventFilter {
    pub key_map: KeyMap,
    pub inputs: Inputs,
}

impl InputEventFilter {
    pub fn new() -> InputResult<Self> {
        Ok(InputEventFilter {
            key_map: KeyMap::from_file("keymap.json")?,
            inputs: Inputs::new(),
        })
    }

    pub fn could_handle(
        &mut self,
        event: &Event<()>,
        own_window_id: WindowId,
        keyboard_focus: bool,
    ) -> bool {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                self.inputs.mouse_motion = Some(PhysicalPosition::new(
                    self.inputs.mouse_motion.unwrap_or(VEC64_ZERO).x + delta.0,
                    self.inputs.mouse_motion.unwrap_or(VEC64_ZERO).y + delta.1,
                ));
            }
            Event::WindowEvent { window_id, event } if own_window_id == *window_id => match event {
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(x, y),
                    ..
                } => {
                    self.inputs.mouse_wheel = Some(PhysicalPosition::new(
                        self.inputs.mouse_wheel.unwrap_or(VEC32_ZERO).x + *x,
                        self.inputs.mouse_wheel.unwrap_or(VEC32_ZERO).y + *y,
                    ))
                }
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::PixelDelta(delta),
                    ..
                } => {
                    self.inputs.mouse_wheel = Some(PhysicalPosition::new(
                        self.inputs.mouse_wheel.unwrap_or(VEC32_ZERO).x
                            + delta.x as f32
                                * self.key_map.touchpad_scroll_sensitivity
                                * if self.key_map.touchpad_invert_x {
                                    -1.
                                } else {
                                    1.
                                },
                        self.inputs.mouse_wheel.unwrap_or(VEC32_ZERO).y
                            - delta.y as f32
                                * self.key_map.touchpad_scroll_sensitivity
                                * if self.key_map.touchpad_invert_y {
                                    -1.
                                } else {
                                    1.
                                },
                    ))
                }
                // unfocused
                WindowEvent::Focused(focused) => {
                    if !focused {
                        for key in [
                            &mut self.inputs.forward,
                            &mut self.inputs.backwards,
                            &mut self.inputs.right,
                            &mut self.inputs.left,
                            &mut self.inputs.up,
                            &mut self.inputs.down,
                        ] {
                            *key = false
                        }
                    }

                    return false;
                }

                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            state,
                            ..
                        },
                    ..
                } if state.is_pressed() => {
                    self.inputs.pause = true;
                }

                WindowEvent::KeyboardInput { event, .. } => {
                    if !keyboard_focus {
                        return true;
                    }

                    let key_code = match event.physical_key {
                        PhysicalKey::Code(key_code) => key_code,
                        _ => return false,
                    };
                    let is_pressed = event.state.is_pressed();

                    match key_code {
                        KeyCode::Escape if is_pressed => {
                            self.inputs.pause = true;
                            return true;
                        }
                        KeyCode::KeyR if is_pressed => {
                            self.inputs.remesh = true;
                            return true;
                        }
                        KeyCode::Digit2 if is_pressed => {
                            self.inputs.lod_up = true;
                            return true;
                        }
                        KeyCode::Digit1 if is_pressed => {
                            self.inputs.lod_down = true;
                            return true;
                        }
                        KeyCode::KeyP if is_pressed => {
                            self.inputs.status = true;
                            return true;
                        }
                        KeyCode::KeyT if is_pressed => {
                            self.inputs.toggle_impl = true;
                            return true;
                        }

                        KeyCode::Space if !is_pressed => {
                            self.inputs.space.release();
                        }

                        KeyCode::Space => {
                            if self.inputs.space.released_for(0.0..0.5) {
                                self.inputs.free_cam = true;
                                println!("space double pressed!")
                            }

                            self.inputs.space.press();
                        }
                        _ => {}
                    }

                    let movement_key = match key_code {
                        _ if key_code == self.key_map.forward => &mut self.inputs.forward,
                        _ if key_code == self.key_map.backwards => &mut self.inputs.backwards,
                        _ if key_code == self.key_map.left => &mut self.inputs.left,
                        _ if key_code == self.key_map.right => &mut self.inputs.right,
                        _ if key_code == self.key_map.up => &mut self.inputs.up,
                        _ if key_code == self.key_map.down => &mut self.inputs.down,
                        _ => return false,
                    };

                    *movement_key = if is_pressed { true } else { false }
                }
                _ => return false,
            },
            _ => return false,
        }
        true
    }

    pub fn get(&self) -> &Inputs {
        &self.inputs
    }

    pub fn frame_done(&mut self) {
        self.inputs.mouse_motion = None;
        self.inputs.mouse_wheel = None;

        self.inputs.pause = false;
        self.inputs.remesh = false;
        self.inputs.lod_up = false;
        self.inputs.lod_down = false;
        self.inputs.free_cam = false;
        self.inputs.status = false;
        self.inputs.toggle_impl = false;

        self.inputs.space.frame_done();
    }
}
