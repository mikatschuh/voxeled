use winit::{
    dpi::PhysicalSize,
    window::{self, WindowId},
};

/// Ein Wrapper für das winit::window::Window, welches das fokussieren und ändern des GrabMode kapselt.
pub struct Window<'a> {
    pub(super) window: &'a window::Window,
    focused: bool,
    size: PhysicalSize<u32>,
    /// Breite / Höhe
    pub aspect_ratio: f32,
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
        let size = window.inner_size();
        Self {
            focused,
            size,
            aspect_ratio: size.width as f32 / size.height as f32,
            window,
        }
    }
    pub fn focused(&self) -> bool {
        self.focused
    }
    /// Eine Methode die den Focus verstellt
    pub fn set_focus(&mut self, focused: bool) {
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

    /// Setzt die Fenstergröße auf einen neuen Wert.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.aspect_ratio = new_size.width as f32 / new_size.height as f32;
        self.size = new_size;
    }
    /// Wrapper für winit::window::Window::id().
    pub fn id(&self) -> WindowId {
        self.window.id()
    }
    /// Wrapper für winit::window::Window::request_redraw().
    pub fn request_redraw(&self) {
        self.window.request_redraw()
    }
}
