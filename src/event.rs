#[cfg(feature = "raw_input")]
use crate::raw_input;
use crate::{device::*, geometry::*, ime::*, window::Window};
use std::path::Path;

/// Trait that must implements for handling events.
pub trait EventHandler {
    /// This is called when there are no events.
    ///
    /// only passed `RunType::Idle` to `Context::run`.
    fn idle(&mut self) {}

    /// This is called at the beginning of a frame.
    ///
    /// only passed `RunType::Idle` to `Context::run`.
    fn begin_frame(&mut self) {}

    /// This is called at the end of a frame.
    ///
    /// only passed `RunType::Idle` to `Context::run`.
    fn end_frame(&mut self) {}

    /// This is called when the window needs redrawing.
    fn draw(&mut self, _: &Window) {}

    /// This is called when the window has been activated.
    fn activated(&mut self, _: &Window) {}

    /// This is called when the window has been inactivated.
    fn inactivated(&mut self, _: &Window) {}

    /// This is called when the window has been closed.
    fn closed(&mut self, _: &Window) {}

    /// This is called when the window has been moved.
    fn moved(&mut self, _: &Window, _: ScreenPosition) {}

    /// This is called when the window is resizing.
    fn resizing(&mut self, _: &Window, _: PhysicalSize<u32>) {}

    /// This is called when the window has been resized.
    fn resized(&mut self, _: &Window, _: PhysicalSize<u32>) {}

    /// This is called when the window's DPI has been changed.
    fn dpi_changed(&mut self, _: &Window) {}

    /// This is called when the mouse button has been pressed and released on the window.
    fn mouse_input(&mut self, _: &Window, _: MouseButton, _: KeyState, _: MouseState) {}

    /// This is called when the cursor has been moved on the window.
    fn cursor_moved(&mut self, _: &Window, _: MouseState) {}

    /// This is called when the cursor has been entered the window.
    fn cursor_entered(&mut self, _: &Window, _: MouseState) {}

    /// This is called when the cursor has been leaved the window.
    fn cursor_leaved(&mut self, _: &Window, _: MouseState) {}

    /// This is called when the keyboard key has been pressed and released.
    fn key_input(&mut self, _: &Window, _: KeyCode, _: KeyState, _: bool) {}

    /// This is called when the keyboard key has been inputed the character.
    fn char_input(&mut self, _: &Window, _: char) {}

    /// This is called when the IME starts composition.
    fn ime_start_composition(&mut self, _: &Window) {}

    /// This is called when the IME composition status has been changed.
    fn ime_composition(&mut self, _: &Window, _: &Composition, _: Option<&CandidateList>) {}

    /// This is called when the IME ends composition.
    fn ime_end_composition(&mut self, _: &Window, _: Option<&str>) {}

    /// This is called when files have been dropped on the window.
    fn drop_files(&mut self, _: &Window, _: &[&Path], _: PhysicalPosition<f32>) {}

    /// This is called when has been inputed raw data.
    #[cfg(feature = "raw_input")]
    fn raw_input(&mut self, _: &Window, _: &raw_input::InputData) {}

    #[cfg(feature = "raw_input")]
    fn raw_input_device_change(
        &mut self,
        _: &Window,
        _: &raw_input::Device,
        _: raw_input::DeviceChangeState,
    ) {
    }
}
