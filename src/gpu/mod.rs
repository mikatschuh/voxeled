mod render;
pub mod state;
use winit::window;

struct Window<'a> {
    focused: bool,
    window: &'a window::Window,
}
impl<'a> Window<'a> {
    fn new(window: &'a window::Window) -> Window<'a> {
        window.set_cursor_visible(false);
        window
            .set_cursor_grab(window::CursorGrabMode::Locked)
            .unwrap();
        Self {
            focused: true,
            window: window,
        }
    }
    fn flip_focus(&mut self) {
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
