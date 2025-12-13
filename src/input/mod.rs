use std::{
    array::IntoIter,
    fs::File,
    io::Read,
    mem,
    ops::{Range, Sub},
    thread::sleep,
    time::{Duration, Instant},
};

use serde::Deserialize;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, Event, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};

use crate::input::error::InputResult;

mod error;

/// Enthält den Zustand einer Taste.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FrameState {
    Pressed,
    JustPressed,

    NotPressed,
    JustReleased,
}

impl From<FrameState> for bool {
    fn from(state: FrameState) -> Self {
        match state {
            FrameState::Pressed | FrameState::JustPressed => true,
            FrameState::NotPressed | FrameState::JustReleased => false,
        }
    }
}

impl Sub for FrameState {
    type Output = f32;
    fn sub(self, other: Self) -> Self::Output {
        self as u32 as f32 - other as u32 as f32
    }
}

pub struct InputState {
    pub frame_state: FrameState,
    pub timestamp: Instant,
}

#[allow(dead_code)]
impl InputState {
    /// Methode die zurück gibt ob eine Taste in diesem Frame gedrückt wurde.
    /// Die Methode gibt nur wahr zurück wenn die Taste im vorherigen Frame nicht gedrückt wurde.
    pub fn just_pressed(&self) -> bool {
        self.frame_state == FrameState::JustPressed
    }

    /// Methode die zurückgibt ob die Taste gerade gedrückt ist.
    pub fn pressed(&self) -> bool {
        self.frame_state.into()
    }

    /// Methode die zurück gibt ob eine Taste in diesem Frame losgelassen wurde.
    pub fn just_released(&self) -> bool {
        self.frame_state == FrameState::JustReleased
    }

    /// Methode die die Zeit in Nanosekunden zurückgibt die die Taste aktuell schon gedrückt wurde.
    pub fn time_pressed(&self) -> Option<u128> {
        if self.frame_state.into() {
            Some(self.timestamp.elapsed().as_nanos())
        } else {
            None
        }
    }

    pub fn time_input_present(&self) -> f32 {
        if self.frame_state.into() {
            self.timestamp.elapsed().as_secs_f32()
        } else {
            0.0
        }
    }

    /// Methode die die Zeit in Nanosekunden zurückgibt die die Taste jetzt schon losgelassen wurde.
    pub fn time_released(&self) -> Option<u128> {
        if !<FrameState as Into<bool>>::into(self.frame_state) {
            Some(self.timestamp.elapsed().as_nanos())
        } else {
            None
        }
    }

    /// Methode die überprüft ob sich die Zeit die eine Taste schon gedrückt wurde in einem gegebenen Bereich befindet.
    pub fn pressed_for(&self, time: Range<u128>) -> bool {
        if <FrameState as Into<bool>>::into(self.frame_state) {
            let time_pressed = self.timestamp.elapsed().as_nanos();
            time.start <= time_pressed && time_pressed <= time.end
        } else {
            false
        }
    }

    /// Methode die überprüft ob sich die Zeit die eine Taste schon nicht mehr gedrückt wurde
    /// in einem gegebenen Bereich befindet.
    pub fn released_for(&self, time: Range<u128>) -> bool {
        if !<FrameState as Into<bool>>::into(self.frame_state) {
            let time_released = self.timestamp.elapsed().as_nanos();
            time.start <= time_released && time_released <= time.end
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub enum DownTime {
    #[default]
    Nothing,
    Instant(Instant),
    Duration(Duration),
    DurationAndInstant {
        duration: Duration,
        instant: Instant,
    },
}

impl DownTime {
    pub fn process(&mut self) -> f32 {
        let secs = match self {
            Self::Nothing => 0.0,
            Self::Instant(instant) => {
                let secs = instant.elapsed().as_secs_f32();
                *instant = Instant::now();
                secs
            }
            Self::Duration(duration) => {
                let secs = duration.as_secs_f32();
                *self = Self::Nothing;
                secs
            }
            Self::DurationAndInstant { duration, instant } => {
                let secs = duration.as_secs_f32() + instant.elapsed().as_secs_f32();
                *self = Self::Instant(Instant::now());
                secs
            }
        };
        secs
    }

    fn press(&mut self) {
        match self {
            Self::Nothing => *self = Self::Instant(Instant::now()),
            Self::Instant(..) => {}
            Self::Duration(duration) => {
                *self = Self::DurationAndInstant {
                    duration: *duration,
                    instant: Instant::now(),
                }
            }
            Self::DurationAndInstant { .. } => {}
        }
    }

    fn release(&mut self) {
        match self {
            Self::Nothing => {}
            Self::Instant(instant) => *self = Self::Duration(instant.elapsed()),
            Self::Duration(..) => {}
            Self::DurationAndInstant {
                duration: dur,
                instant,
            } => *self = Self::Duration(*dur + instant.elapsed()),
        }
    }
}

const VEC32_ZERO: PhysicalPosition<f32> = PhysicalPosition::new(0., 0.);
const VEC64_ZERO: PhysicalPosition<f64> = PhysicalPosition::new(0., 0.);

#[derive(Clone, Default, Debug)]
pub struct Inputs {
    pub forward: DownTime,
    pub backwards: DownTime,
    pub left: DownTime,
    pub right: DownTime,
    pub up: DownTime,
    pub down: DownTime,

    pub mouse_motion: Option<PhysicalPosition<f64>>,
    pub mouse_wheel: Option<PhysicalPosition<f32>>,

    pub esc: bool,
}

impl Inputs {
    fn every_downtime(&mut self) -> IntoIter<&mut DownTime, 6> {
        [
            &mut self.forward,
            &mut self.backwards,
            &mut self.left,
            &mut self.right,
            &mut self.up,
            &mut self.down,
        ]
        .into_iter()
    }
}

/*#[derive(Deserialize)]
pub struct KeyMap {
    pub forward: char,
    pub backwards: char,
    pub left: char,
    pub right: char,
    pub up: char,
    pub down: char,

    touchpad_sensitivity: f32,
}*/

pub struct InputEventFilter {
    // key_map: KeyMap,
    pub inputs: Inputs,
}

impl InputEventFilter {
    pub fn new() -> InputResult<Self> {
        /*let mut settings = File::open("settings.json")?;
        let mut json_settings = String::new();
        settings.read_to_string(&mut json_settings);
        let key_map: KeyMap = serde_json::from_str(&json_settings)?;
        */

        Ok(InputEventFilter {
            // key_map,
            inputs: Inputs::default(),
        })
    }

    pub fn could_handle(&mut self, event: &Event<()>, own_window_id: WindowId) -> bool {
        match event {
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    self.inputs.mouse_motion = Some(PhysicalPosition::new(
                        self.inputs.mouse_motion.unwrap_or(VEC64_ZERO).x + delta.0,
                        self.inputs.mouse_motion.unwrap_or(VEC64_ZERO).y - delta.1,
                    ));
                }
                _ => return false,
            },
            Event::WindowEvent { window_id, event } if own_window_id == *window_id => match event {
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.inputs.mouse_wheel = Some(PhysicalPosition::new(
                            self.inputs.mouse_wheel.unwrap_or(VEC32_ZERO).x + *x,
                            self.inputs.mouse_wheel.unwrap_or(VEC32_ZERO).y + *y,
                        ));
                    }
                    MouseScrollDelta::PixelDelta(delta) => {
                        self.inputs.mouse_wheel = Some(PhysicalPosition::new(
                            self.inputs.mouse_wheel.unwrap_or(VEC32_ZERO).x + delta.x as f32, //* self.key_map.touchpad_sensitivity,
                            self.inputs.mouse_wheel.unwrap_or(VEC32_ZERO).y - delta.y as f32, // * self.key_map.touchpad_sensitivity,
                        ))
                    }
                },
                // unfocused
                WindowEvent::Focused(focused) if !focused => {
                    self.inputs
                        .every_downtime()
                        .for_each(|down_time| down_time.release());

                    return false;
                }
                WindowEvent::KeyboardInput { event, .. }
                    if event.physical_key == PhysicalKey::Code(KeyCode::Escape)
                        && event.state.is_pressed() =>
                {
                    self.inputs.esc = true
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    let key_code = match event.physical_key {
                        PhysicalKey::Code(key_code) => key_code,
                        _ => return false,
                    };

                    let down_time = match key_code {
                        KeyCode::KeyW => &mut self.inputs.forward,
                        KeyCode::KeyS => &mut self.inputs.backwards,
                        KeyCode::KeyA => &mut self.inputs.left,
                        KeyCode::KeyD => &mut self.inputs.right,
                        KeyCode::Space => &mut self.inputs.up,
                        KeyCode::ShiftLeft => &mut self.inputs.down,
                        _ => return false,
                    };

                    if event.state.is_pressed() {
                        down_time.press();
                    } else {
                        down_time.release();
                    }
                }
                _ => return false,
            },
            _ => return false,
        }
        true
    }

    pub fn get(&mut self) -> &mut Inputs {
        &mut self.inputs
    }

    pub fn frame_done(&mut self) {
        self.inputs.mouse_motion = None;
        self.inputs.mouse_wheel = None;

        self.inputs.esc = false;
    }
}
