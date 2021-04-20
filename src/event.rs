#![allow(unused_variables)]

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

    /// This is called before a event.
    ///
    /// only passed `RunType::Idle` to `Context::run`.
    fn pre_processing(&mut self) {}

    /// This is called after a event.
    ///
    /// only passed `RunType::Idle` to `Context::run`.
    fn post_processing(&mut self) {}

    /// This is called when the window needs redrawing.
    fn draw(&mut self, window: &Window) {}

    /// This is called when the window has been activated.
    fn activated(&mut self, window: &Window) {}

    /// This is called when the window has been inactivated.
    fn inactivated(&mut self, window: &Window) {}

    /// This is called when the window has been closed.
    fn closed(&mut self, window: &Window) {}

    /// This is called when the window has been moved.
    fn moved(&mut self, window: &Window, position: ScreenPosition) {}

    /// This is called when the window is resizing.
    fn resizing(&mut self, window: &Window, size: PhysicalSize<u32>) {}

    /// This is called when the window has been resized.
    fn resized(&mut self, window: &Window, size: PhysicalSize<u32>) {}

    /// This is called when the window's DPI has been changed.
    fn dpi_changed(&mut self, window: &Window) {}

    /// This is called when the mouse button has been pressed and released on the window.
    fn mouse_input(
        &mut self,
        window: &Window,
        button: MouseButton,
        state: KeyState,
        mouse_state: MouseState,
    ) {
    }

    /// This is called when the cursor has been moved on the window.
    fn cursor_moved(&mut self, window: &Window, mouse_state: MouseState) {}

    /// This is called when the cursor has been entered the window.
    fn cursor_entered(&mut self, window: &Window, mouse_state: MouseState) {}

    /// This is called when the cursor has been leaved the window.
    fn cursor_leaved(&mut self, window: &Window, mouse_state: MouseState) {}

    /// This is called when the keyboard key has been pressed and released.
    fn key_input(
        &mut self,
        window: &Window,
        key_code: KeyCode,
        state: KeyState,
        prev_pressed: bool,
    ) {
    }

    /// This is called when the keyboard key has been inputed the character.
    fn char_input(&mut self, window: &Window, c: char) {}

    /// This is called when the IME starts composition.
    fn ime_start_composition(&mut self, window: &Window) {}

    /// This is called when the IME composition status has been changed.
    fn ime_composition(
        &mut self,
        window: &Window,
        composition: &Composition,
        candidate_list: Option<&CandidateList>,
    ) {
    }

    /// This is called when the IME ends composition.
    fn ime_end_composition(&mut self, window: &Window, result_string: Option<&str>) {}

    /// This is called when files have been dropped on the window.
    fn drop_files(&mut self, window: &Window, paths: &[&Path], position: PhysicalPosition<f32>) {}

    /// This is called when raw data has been inputed.
    #[cfg(feature = "raw_input")]
    fn raw_input(&mut self, window: &Window, data: &raw_input::InputData) {}

    /// This is called when a device state has been changead.
    #[cfg(feature = "raw_input")]
    fn raw_input_device_change(
        &mut self,
        window: &Window,
        device: &raw_input::Device,
        state: raw_input::DeviceChangeState,
    ) {
    }
}
