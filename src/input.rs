/// Enthält den Zustand einer Taste.
#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub enum State {
    Pressed,
    JustPressed,
    #[default]
    NotPressed,
    JustReleased,
}
impl From<State> for bool {
    fn from(state: State) -> Self {
        (state as u8) < 2
    }
}
use std::ops::Sub;
impl Sub for State {
    type Output = i32;
    fn sub(self, other: Self) -> Self::Output {
        self as i32 - other as i32
    }
}
impl State {
    /// Funktion die zurück gibt ob ein Zustand JustPressed ist.
    pub fn just_pressed(&self) -> bool {
        *self == State::JustPressed
    }
}

/// Eine Struktur die eine Karte aller relevanten Tasten und ihren Zustand speichert.
#[derive(Default)]
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

    pub mouse_motion: Option<(f64, f64)>,
    pub mouse_wheel: Option<f32>,
}
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};
impl Keys {
    /// Funktion die true zurückgibt wenn das Event ein Input war und die Karte aller relevanten Tasten aktualisiert.
    /// Sie gibt false zurück wenn sie das Event nicht handeln konnte. Die Funktion funktioniert also wie ein Sieb,
    /// welches Tastatur / Maus - Events rausfiltert.
    pub fn handled_event(&mut self, own_window_id: WindowId, event: &Event<()>) -> bool {
        match event {
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    self.mouse_motion = Some(*delta);
                }
                DeviceEvent::MouseWheel { delta } => match delta {
                    MouseScrollDelta::LineDelta(_, y) => self.mouse_wheel = Some(*y),
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
                    match pressed_key {
                        State::Pressed => {
                            if event.state == ElementState::Released {
                                *pressed_key = State::JustReleased
                            }
                        }
                        State::NotPressed => {
                            if event.state == ElementState::Pressed {
                                *pressed_key = State::JustPressed
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
            if let State::JustPressed = key {
                *key = State::Pressed
            } else if let State::JustReleased = key {
                *key = State::NotPressed
            }
        }
        if let Some(..) = self.mouse_motion {
            self.mouse_motion = None
        }
        if let Some(..) = self.mouse_wheel {
            self.mouse_wheel = None
        }
    }
}
