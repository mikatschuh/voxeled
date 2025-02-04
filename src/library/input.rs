use std::{ops::Range, time::Instant};
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};

/// Enthält den Zustand einer Taste.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InputState {
    Pressed,
    JustPressed,

    NotPressed,
    JustReleased,
}
impl From<InputState> for bool {
    fn from(state: InputState) -> Self {
        match state {
            InputState::Pressed | InputState::JustPressed => true,
            InputState::NotPressed | InputState::JustReleased => false,
        }
    }
}
use std::ops::Sub;
impl Sub for InputState {
    type Output = i32;
    fn sub(self, other: Self) -> Self::Output {
        self as i32 - other as i32
    }
}

pub struct State {
    pub state: InputState,
    pub since: Instant,
}
impl State {
    /// Methode die zurück gibt ob eine Taste in diesem Frame gedrückt wurde.
    /// Die Methode gibt nur wahr zurück wenn die Taste im vorherigen Frame nicht gedrückt wurde.
    pub fn just_pressed(&self) -> bool {
        self.state == InputState::JustPressed
    }
    /// Methode die zurück gibt ob eine Taste in diesem Frame losgelassen wurde.
    pub fn just_released(&self) -> bool {
        self.state == InputState::JustReleased
    }
    /// Methode die zurückgibt ob die Taste gerade gedrückt ist.
    pub fn pressed(&self) -> bool {
        self.state.into()
    }
    /// Methode die die Zeit in Nanosekunden zurückgibt die die Taste aktuell schon gedrückt wurde.
    pub fn time_pressed(&self) -> Option<u128> {
        if self.state.into() {
            Some(self.since.elapsed().as_nanos())
        } else {
            None
        }
    }
    /// Methode die die Zeit in Nanosekunden zurückgibt die die Taste jetzt schon losgelassen wurde.
    pub fn time_released(&self) -> Option<u128> {
        if !<InputState as Into<bool>>::into(self.state) {
            Some(self.since.elapsed().as_nanos())
        } else {
            None
        }
    }
    /// Methode die überprüft ob sich die Zeit die eine Taste schon gedrückt wurde in einem gegebenen Bereich befindet.
    pub fn pressed_for(&self, time: Range<u128>) -> bool {
        if let InputState::Pressed | InputState::JustPressed = self.state {
            let time_pressed = self.since.elapsed().as_nanos();
            time.start <= time_pressed && time_pressed <= time.end
        } else {
            false
        }
    }
    /// Methode die überprüft ob sich die Zeit die eine Taste schon nicht mehr gedrückt wurde
    /// in einem gegebenen Bereich befindet.
    pub fn released_for(&self, time: Range<u128>) -> bool {
        if let InputState::NotPressed | InputState::JustReleased = self.state {
            let time_released = self.since.elapsed().as_nanos();
            time.start <= time_released && time_released <= time.end
        } else {
            false
        }
    }
}
/// Eine Struktur die eine Karte aller relevanten Tasten und ihren Zustand speichert.
pub struct Keys {
    pub w: State,
    pub a: State,
    pub s: State,
    pub d: State,

    pub e: State,
    pub q: State,
    pub f: State,

    pub space: State,
    pub shift: State,
    pub esc: State,

    pub mouse_motion: (f64, f64),
    pub mouse_wheel: f32,
}
impl Default for Keys {
    fn default() -> Self {
        Self {
            w: State {
                state: InputState::NotPressed,
                since: Instant::now(),
            },
            a: State {
                state: InputState::NotPressed,
                since: Instant::now(),
            },
            s: State {
                state: InputState::NotPressed,
                since: Instant::now(),
            },
            d: State {
                state: InputState::NotPressed,
                since: Instant::now(),
            },

            e: State {
                state: InputState::NotPressed,
                since: Instant::now(),
            },
            q: State {
                state: InputState::NotPressed,
                since: Instant::now(),
            },
            f: State {
                state: InputState::NotPressed,
                since: Instant::now(),
            },

            space: State {
                state: InputState::NotPressed,
                since: Instant::now(),
            },
            shift: State {
                state: InputState::NotPressed,
                since: Instant::now(),
            },
            esc: State {
                state: InputState::NotPressed,
                since: Instant::now(),
            },

            mouse_motion: (0.0, 0.0),
            mouse_wheel: 0.0,
        }
    }
}

impl Keys {
    /// Erstellt eine neue Keys Instanz.
    pub fn new() -> Self {
        Self::default()
    }
    /// Funktion die true zurückgibt wenn das Event ein Input war und die Karte aller relevanten Tasten aktualisiert.
    /// Sie gibt false zurück wenn sie das Event nicht handeln konnte. Die Funktion funktioniert also wie ein Sieb,
    /// welches Tastatur / Maus - Events rausfiltert.
    pub fn handled_event(&mut self, own_window_id: WindowId, event: &Event<()>) -> bool {
        match event {
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    self.mouse_motion = *delta;
                }
                DeviceEvent::MouseWheel { delta } => match delta {
                    MouseScrollDelta::LineDelta(_, y) => self.mouse_wheel = *y,
                    _ => return false,
                },
                _ => return false,
            },
            Event::WindowEvent { window_id, event } if *window_id == own_window_id => match event {
                WindowEvent::KeyboardInput { event, .. } => {
                    let pressed_key: &mut State;
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyW) => pressed_key = &mut self.w,
                        PhysicalKey::Code(KeyCode::KeyA) => pressed_key = &mut self.a,

                        PhysicalKey::Code(KeyCode::KeyS) => pressed_key = &mut self.s,
                        PhysicalKey::Code(KeyCode::KeyD) => pressed_key = &mut self.d,

                        PhysicalKey::Code(KeyCode::KeyE) => pressed_key = &mut self.e,
                        PhysicalKey::Code(KeyCode::KeyQ) => pressed_key = &mut self.q,
                        PhysicalKey::Code(KeyCode::KeyF) => pressed_key = &mut self.f,

                        PhysicalKey::Code(KeyCode::Space) => pressed_key = &mut self.space,
                        PhysicalKey::Code(KeyCode::ShiftLeft) => pressed_key = &mut self.shift,
                        PhysicalKey::Code(KeyCode::Escape) => pressed_key = &mut self.esc,

                        _ => return false,
                    }
                    match pressed_key.state {
                        InputState::Pressed { .. } => {
                            if event.state == ElementState::Released {
                                *pressed_key = State {
                                    state: InputState::JustReleased,
                                    since: Instant::now(),
                                }
                            }
                        }
                        InputState::NotPressed { .. } => {
                            if event.state == ElementState::Pressed {
                                *pressed_key = State {
                                    state: InputState::JustPressed,
                                    since: Instant::now(),
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => return false,
            },
            _ => return false,
        }
        true
    }
    /// Wird nach jedem Frame aufgerufen um die Tasten zu aktualisieren.
    /// z.B. Wenn eine Taste in diesem Frame JustPressed (gerade gedrückt) war,
    /// dann sollte sie im Nächsten Pressed sein (gedrückt).
    pub fn update(&mut self) {
        for key in [
            &mut self.w,
            &mut self.a,
            &mut self.s,
            &mut self.d,
            &mut self.e,
            &mut self.q,
            &mut self.f,
            &mut self.space,
            &mut self.shift,
            &mut self.esc,
        ] {
            if let InputState::JustPressed = key.state {
                key.state = InputState::Pressed;
            } else if let InputState::JustReleased = key.state {
                key.state = InputState::NotPressed;
            }
        }
        self.mouse_motion = (0.0, 0.0);

        self.mouse_wheel = 0.0
    }
}
