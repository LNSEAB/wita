use crate::{device::*, window::Window};

pub trait EventHandler {
    fn activated(&mut self, _: &Window) {}
    fn inactivated(&mut self, _: &Window) {}
    fn closed(&mut self, _: &Window) {}
    fn dpi_changed(&mut self, _: &Window) {}
    fn idle(&mut self) {}
    fn mouse_input(&mut self, _: &Window, _: MouseButton, _: KeyState, _: MouseState) {}
}
