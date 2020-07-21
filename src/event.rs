use crate::{device::*, window::Window, ime::*};

pub trait EventHandler {
    fn activated(&mut self, _: &Window) {}
    fn inactivated(&mut self, _: &Window) {}
    fn closed(&mut self, _: &Window) {}
    fn dpi_changed(&mut self, _: &Window) {}
    fn idle(&mut self) {}
    fn mouse_input(&mut self, _: &Window, _: MouseButton, _: KeyState, _: MouseState) {}
    fn cursor_moved(&mut self, _: &Window, _: MouseState) {}
    fn cursor_entered(&mut self, _: &Window, _: MouseState) {}
    fn cursor_leaved(&mut self, _: &Window, _: MouseState) {}
    fn key_input(&mut self, _: &Window, _: KeyCode, _: KeyState) {}
    fn char_input(&mut self, _: &Window, _: char) {}
    fn ime_start_composition(&mut self, _: &Window) {}
    fn ime_composition(&mut self, _: &Window, _: &Composition, _: Option<&CandidateList>) {}
    fn ime_end_composition(&mut self, _: &Window, _: Option<&str>) {}
}
