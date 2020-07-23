use crate::geometry::*;
use serde::{Deserialize, Serialize};

/// Describes the state of a keyboard key and a mouse button.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum KeyState {
    Pressed,
    Released,
}

/// Describes mouse buttons.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Ex(u32),
}

/// A mouse cursor position and pressed mouse buttons.
#[derive(Clone, Debug)]
pub struct MouseState<'a> {
    pub position: LogicalPosition<f32>,
    pub buttons: &'a [MouseButton],
}

/// Describes keyboard key names.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum VirtualKey {
    Char(char),
    Esc,
    Tab,
    CapsLock,
    Shift,
    Ctrl,
    Alt,
    BackSpace,
    Enter,
    Space,
    PrintScreen,
    ScrollLock,
    Pause,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    NumLock,
    NumPad(u8),
    NumAdd,
    NumSub,
    NumMul,
    NumDiv,
    NumDecimal,
    F(u8),
    Other(u32),
}

/// A keyboard scan code
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct ScanCode(pub u32);

/// A virtual key and a scan code.
#[derive(Clone, Copy, Debug)]
pub struct KeyCode {
    pub vkey: VirtualKey,
    pub scan_code: ScanCode,
}

impl KeyCode {
    pub fn new(vkey: VirtualKey, scan_code: ScanCode) -> Self {
        Self { vkey, scan_code }
    }
}
