use winit::{dpi::PhysicalSize, window};

/// Ein Wrapper für das winit::window::Window, welches das fokussieren und ändern des GrabMode kapselt.
pub struct Window<'a> {
    pub focused: bool,
    pub size: PhysicalSize<u32>,
    pub window: &'a window::Window,
}
impl<'a> Window<'a> {
    /// Erzeugt eine Window - Instanz wobei der GrabMode an focused anpasst
    /// damit der Zustand der Variable und des Windows konsistent bleibt.
    pub fn from(window: &'a window::Window, focused: bool) -> Window<'a> {
        if focused {
            window.set_cursor_visible(false);
            window
                .set_cursor_grab(window::CursorGrabMode::Locked)
                .unwrap();
        } else {
            window.set_cursor_visible(true);
            window
                .set_cursor_grab(window::CursorGrabMode::None)
                .unwrap();
        }
        Self {
            focused,
            size: window.inner_size(),
            window,
        }
    }
    /// Eine Methode die den Focus verstellt
    pub fn _set_focus(&mut self, focused: bool) {
        if focused {
            self.window.set_cursor_visible(false);
            self.window
                .set_cursor_grab(window::CursorGrabMode::Locked)
                .unwrap();
        } else {
            self.window.set_cursor_visible(true);
            self.window
                .set_cursor_grab(window::CursorGrabMode::None)
                .unwrap();
        }
        self.focused = focused;
    }
    /// Eine Methode die den GrabMode flipped.
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
