use winit::window;

pub struct Window<'a> {
    pub focused: bool,
    pub window: &'a window::Window,
}
impl<'a> Window<'a> {
    pub fn from(window: &'a window::Window, focused: bool) -> Window<'a> {
        if focused {
            window.set_cursor_visible(false);
            window
                .set_cursor_grab(window::CursorGrabMode::Locked)
                .unwrap();
            Self {
                focused,
                window: window,
            }
        } else {
            window.set_cursor_visible(true);
            window
                .set_cursor_grab(window::CursorGrabMode::None)
                .unwrap();
            Self {
                focused,
                window: window,
            }
        }
    }
    pub fn flip_focus(&mut self) {
        match self.focused {
            true => {
                self.window.set_cursor_visible(true);
                self.window
                    .set_cursor_grab(window::CursorGrabMode::None)
                    .unwrap();
            }
            false => {
                self.window.set_cursor_visible(false);
                self.window
                    .set_cursor_grab(window::CursorGrabMode::Locked)
                    .unwrap();
            }
        }
        self.focused = !self.focused
    }
}
