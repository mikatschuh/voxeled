use std::{ops::Range, time::Instant};
use winit::{
    dpi::PhysicalPosition,
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
    type Output = f32;
    fn sub(self, other: Self) -> Self::Output {
        self as u32 as f32 - other as u32 as f32
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
        if <InputState as Into<bool>>::into(self.state) {
            let time_pressed = self.since.elapsed().as_nanos();
            time.start <= time_pressed && time_pressed <= time.end
        } else {
            false
        }
    }
    /// Methode die überprüft ob sich die Zeit die eine Taste schon nicht mehr gedrückt wurde
    /// in einem gegebenen Bereich befindet.
    pub fn released_for(&self, time: Range<u128>) -> bool {
        if !<InputState as Into<bool>>::into(self.state) {
            let time_released = self.since.elapsed().as_nanos();
            time.start <= time_released && time_released <= time.end
        } else {
            false
        }
    }
}
struct KeyEventState {
    pressed_in_the_last_frame: bool,
    pressed_in_this_frame: bool,
    last_change: Instant,
}
impl KeyEventState {
    fn new(now: Instant) -> Self {
        Self {
            pressed_in_the_last_frame: false,
            pressed_in_this_frame: false,
            last_change: now,
        }
    }
}
/// Ein Objekt welches Events nach KeyEvents filtern kann.
pub struct InputEventFilter {
    w: KeyEventState,
    a: KeyEventState,
    s: KeyEventState,
    d: KeyEventState,

    e: KeyEventState,
    q: KeyEventState,
    f: KeyEventState,

    p: KeyEventState,

    space: KeyEventState,
    shift: KeyEventState,
    esc: KeyEventState,

    mouse_motion: PhysicalPosition<f64>,
    mouse_wheel: PhysicalPosition<f32>,

    /// Two space clicks in 40 ms.
    space_double_tap: Vec<Instant>,
}
impl InputEventFilter {
    /// Erstellt einen neuen Input Event Filter.
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            w: KeyEventState::new(now),
            a: KeyEventState::new(now),
            s: KeyEventState::new(now),
            d: KeyEventState::new(now),

            e: KeyEventState::new(now),
            q: KeyEventState::new(now),
            f: KeyEventState::new(now),

            p: KeyEventState::new(now),

            space: KeyEventState::new(now),
            shift: KeyEventState::new(now),
            esc: KeyEventState::new(now),

            mouse_motion: PhysicalPosition::new(0.0, 0.0),
            mouse_wheel: PhysicalPosition::new(0.0, 0.0),

            space_double_tap: vec![],
        }
    }
    /// Funktion die true zurückgibt wenn das Event ein Input war der abgegriffen wurde.
    /// Sie gibt false zurück wenn sie das Event nicht handeln konnte. Die Funktion funktioniert also wie ein Sieb,
    /// welches Tastatur / Maus - Events rausfiltert.
    pub fn handled_event(&mut self, event: &Event<()>, own_window_id: WindowId) -> bool {
        match event {
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    self.mouse_motion = PhysicalPosition::new(
                        self.mouse_motion.x + delta.0,
                        self.mouse_motion.y - delta.1,
                    );
                }
                _ => return false,
            },
            Event::WindowEvent { window_id, event } if own_window_id == *window_id => match event {
                WindowEvent::Focused(focused) if !focused => {
                    [
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
                        &mut self.p,
                    ]
                    .into_iter()
                    .for_each(|input| input.pressed_in_this_frame = false);
                    self.mouse_motion = PhysicalPosition::new(0.0, 0.0);
                    self.mouse_wheel = PhysicalPosition::new(0.0, 0.0);
                    self.space_double_tap = vec![];
                    return false;
                }
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.mouse_wheel =
                            PhysicalPosition::new(self.mouse_wheel.x + *x, self.mouse_wheel.y - *y)
                    }
                    MouseScrollDelta::PixelDelta(delta) => {
                        self.mouse_wheel = PhysicalPosition::new(
                            self.mouse_wheel.x + delta.x as f32,
                            self.mouse_wheel.y - delta.y as f32,
                        )
                    }
                },
                WindowEvent::KeyboardInput { event, .. } => {
                    let pressed_key: &mut KeyEventState;

                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyW) => pressed_key = &mut self.w,
                        PhysicalKey::Code(KeyCode::KeyA) => pressed_key = &mut self.a,

                        PhysicalKey::Code(KeyCode::KeyS) => pressed_key = &mut self.s,
                        PhysicalKey::Code(KeyCode::KeyD) => pressed_key = &mut self.d,

                        PhysicalKey::Code(KeyCode::KeyE) => pressed_key = &mut self.e,
                        PhysicalKey::Code(KeyCode::KeyQ) => pressed_key = &mut self.q,
                        PhysicalKey::Code(KeyCode::KeyF) => pressed_key = &mut self.f,

                        PhysicalKey::Code(KeyCode::KeyP) => pressed_key = &mut self.p,

                        PhysicalKey::Code(KeyCode::Space) => {
                            self.space_double_tap_cleanup();
                            self.space_double_tap.push(Instant::now());
                            pressed_key = &mut self.space
                        }
                        PhysicalKey::Code(KeyCode::ShiftLeft) => pressed_key = &mut self.shift,
                        PhysicalKey::Code(KeyCode::Escape) => pressed_key = &mut self.esc,

                        _ => return false,
                    }
                    match pressed_key.pressed_in_this_frame {
                        true if event.state == ElementState::Released => {
                            pressed_key.pressed_in_this_frame = false;
                            pressed_key.last_change = Instant::now()
                        }
                        false => match event.state {
                            ElementState::Pressed => {
                                pressed_key.pressed_in_this_frame = true;
                                pressed_key.last_change = Instant::now()
                            }
                            ElementState::Released => {
                                if pressed_key.pressed_in_the_last_frame {
                                    pressed_key.last_change = Instant::now()
                                }
                            }
                        },
                        _ => {}
                    }
                }
                _ => return false,
            },
            _ => return false,
        }
        true
    }
    /// Eine Funktion die benutzt werden kann um eine Karte aller relevanten Tasten zu erhalten.
    /// # Beispiel
    /// ```
    /// let input_event_filter = InputEventFilter::new();
    /// input_event_filter.handled_event(Event::WindowEvent {
    ///     event: WindowEvent::KeyboardInput {
    ///         event: KeyEvent {
    ///             state: ElementState::Pressed,
    ///             physical_key: PhysicalKey::Code(KeyCode::KeyW),
    ///             ...
    ///         },
    ///         ...
    ///     },
    ///     ...
    /// });
    /// let key_map = input_event_filter.get();
    /// assert_eq!(key_map.w.pressed(), true);
    /// ```
    pub fn get(&mut self) -> KeyMap {
        use InputState::*;
        let mut key_map = KeyMap::default();
        [
            (&mut self.w, &mut key_map.w),
            (&mut self.a, &mut key_map.a),
            (&mut self.s, &mut key_map.s),
            (&mut self.d, &mut key_map.d),
            (&mut self.e, &mut key_map.e),
            (&mut self.q, &mut key_map.q),
            (&mut self.f, &mut key_map.f),
            (&mut self.p, &mut key_map.p),
            (&mut self.space, &mut key_map.space),
            (&mut self.shift, &mut key_map.shift),
            (&mut self.esc, &mut key_map.esc),
        ]
        .into_iter()
        .for_each(
            |(input, final_state)| match input.pressed_in_the_last_frame {
                true => match input.pressed_in_this_frame {
                    true => {
                        *final_state = State {
                            state: Pressed,
                            since: input.last_change,
                        };
                    }
                    false => {
                        *final_state = State {
                            state: JustReleased,
                            since: input.last_change,
                        };
                        input.pressed_in_the_last_frame = false // the last and current frame arent the same, so the last is set to the current
                    }
                },
                false => match input.pressed_in_this_frame {
                    true => {
                        *final_state = State {
                            state: JustPressed,
                            since: input.last_change,
                        };
                        input.pressed_in_the_last_frame = true // the last and current frame arent the same, so the last is set to the current
                    }
                    false => {
                        *final_state = State {
                            state: NotPressed,
                            since: input.last_change,
                        }
                    }
                },
            },
        );
        key_map.mouse_motion = self.mouse_motion;
        key_map.mouse_wheel = self.mouse_wheel;
        self.space_double_tap_cleanup();
        key_map.space_double_tap = self.space_double_tap.len() == 2;
        self.mouse_motion = PhysicalPosition::new(0.0, 0.0);
        self.mouse_wheel = PhysicalPosition::new(0.0, 0.0);
        key_map
    }
    fn space_double_tap_cleanup(&mut self) {
        for i in 0..self.space_double_tap.len() {
            if self.space_double_tap[i].elapsed().as_nanos() > 40_000_000 {
                self.space_double_tap.remove(i);
            }
        }
    }
}
/// Eine Struktur die eine Karte aller relevanten Tasten und ihren Zustand speichert.
pub struct KeyMap {
    pub w: State,
    pub a: State,
    pub s: State,
    pub d: State,

    pub e: State,
    pub q: State,
    pub f: State,

    pub p: State,

    pub space: State,
    pub shift: State,
    pub esc: State,

    pub mouse_motion: PhysicalPosition<f64>,
    pub mouse_wheel: PhysicalPosition<f32>,

    pub space_double_tap: bool,
}
impl Default for KeyMap {
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
            p: State {
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

            mouse_motion: PhysicalPosition::new(0.0, 0.0),
            mouse_wheel: PhysicalPosition::new(0.0, 0.0),

            space_double_tap: false,
        }
    }
}
