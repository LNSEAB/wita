use crate::geometry::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Ex(u32),
}

#[derive(Clone, Debug)]
pub struct MouseState<'a> {
    pub position: LogicalPosition<f32>,
    pub buttons: &'a [MouseButton],
}
