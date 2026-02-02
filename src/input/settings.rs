use std::{fs::File, io::Read, path::Path};

use serde::Deserialize;
use winit::keyboard::KeyCode;

use crate::input::error::InputResult;

pub struct KeyMap {
    pub forward: KeyCode,
    pub backwards: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,

    pub touchpad_scroll_sensitivity: f32,
    pub touchpad_invert_x: bool,
    pub touchpad_invert_y: bool,
}

#[derive(Deserialize)]
struct ReMaps {
    forward: Option<String>,
    backwards: Option<String>,
    right: Option<String>,
    left: Option<String>,
    down: Option<String>,
    up: Option<String>,

    touchpad_scroll_sensitivity: Option<f32>,
    touchpad_invert_x: Option<bool>,
    touchpad_invert_y: Option<bool>,
}

impl KeyMap {
    pub(super) fn from_file(path: impl AsRef<Path>) -> InputResult<KeyMap> {
        let mut settings = File::open(path)?;
        let mut json_settings = String::new();
        settings.read_to_string(&mut json_settings)?;
        let re_maps: ReMaps = serde_json::from_str(&json_settings)?;

        Ok(KeyMap {
            forward: str_to_keycode(re_maps.forward.as_deref().unwrap_or("w"))?,
            backwards: str_to_keycode(re_maps.backwards.as_deref().unwrap_or("s"))?,
            left: str_to_keycode(re_maps.left.as_deref().unwrap_or("a"))?,
            right: str_to_keycode(re_maps.right.as_deref().unwrap_or("d"))?,
            up: str_to_keycode(re_maps.up.as_deref().unwrap_or("Space"))?,
            down: str_to_keycode(re_maps.down.as_deref().unwrap_or("Shift"))?,

            touchpad_scroll_sensitivity: re_maps.touchpad_scroll_sensitivity.unwrap_or(1.),
            touchpad_invert_x: re_maps.touchpad_invert_x.unwrap_or(false),
            touchpad_invert_y: re_maps.touchpad_invert_y.unwrap_or(false),
        })
    }
}

fn str_to_keycode(code: &str) -> InputResult<KeyCode> {
    use KeyCode::*;

    Ok(match code.to_lowercase().as_str() {
        "q" => KeyQ,
        "w" => KeyW,
        "e" => KeyE,
        "r" => KeyR,
        "t" => KeyT,
        "z" => KeyZ,
        "u" => KeyU,
        "i" => KeyI,
        "o" => KeyO,
        "p" => KeyP,

        "a" => KeyA,
        "s" => KeyS,
        "d" => KeyD,
        "f" => KeyF,
        "g" => KeyG,
        "h" => KeyH,
        "j" => KeyJ,
        "k" => KeyK,
        "l" => KeyL,

        "y" => KeyY,
        "x" => KeyX,
        "c" => KeyC,
        "v" => KeyV,
        "b" => KeyB,
        "n" => KeyN,
        "m" => KeyM,

        "0" => Digit0,
        "1" => Digit1,
        "2" => Digit2,
        "3" => Digit3,
        "4" => Digit4,
        "5" => Digit5,
        "6" => Digit6,
        "7" => Digit7,
        "8" => Digit8,
        "9" => Digit9,

        "shift" => ShiftLeft,
        "rshift" => ShiftRight,
        "space" => Space,

        _ => return Err(super::error::InputError::UnknownKeys),
    })
}
