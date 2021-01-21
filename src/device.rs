use crate::geometry::*;
use winapi::um::winuser::*;
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
    pub position: PhysicalPosition<i32>,
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
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    LAlt,
    RAlt,
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

pub(crate) fn as_virtual_key(k: i32) -> VirtualKey {
    const ZERO: i32 = b'0' as i32;
    const Z: i32 = b'Z' as i32;
    let value = k as i32;
    match value {
        v @ ZERO..=Z => VirtualKey::Char((v as u8).into()),
        VK_ESCAPE => VirtualKey::Esc,
        VK_TAB => VirtualKey::Tab,
        VK_CAPITAL => VirtualKey::CapsLock,
        VK_SHIFT => VirtualKey::Shift,
        VK_CONTROL => VirtualKey::Ctrl,
        VK_MENU => VirtualKey::Alt,
        VK_BACK => VirtualKey::BackSpace,
        VK_RETURN => VirtualKey::Enter,
        VK_SPACE => VirtualKey::Space,
        VK_SNAPSHOT => VirtualKey::PrintScreen,
        VK_SCROLL => VirtualKey::ScrollLock,
        VK_PAUSE => VirtualKey::Pause,
        VK_INSERT => VirtualKey::Insert,
        VK_DELETE => VirtualKey::Delete,
        VK_HOME => VirtualKey::Home,
        VK_END => VirtualKey::End,
        VK_PRIOR => VirtualKey::PageUp,
        VK_NEXT => VirtualKey::PageDown,
        VK_UP => VirtualKey::Up,
        VK_DOWN => VirtualKey::Down,
        VK_LEFT => VirtualKey::Left,
        VK_RIGHT => VirtualKey::Right,
        VK_NUMLOCK => VirtualKey::NumLock,
        v @ VK_NUMPAD0..=VK_NUMPAD9 => VirtualKey::NumPad((v - VK_NUMPAD0) as u8),
        VK_ADD => VirtualKey::NumAdd,
        VK_SUBTRACT => VirtualKey::NumSub,
        VK_MULTIPLY => VirtualKey::NumMul,
        VK_DIVIDE => VirtualKey::NumDiv,
        VK_DECIMAL => VirtualKey::NumDecimal,
        VK_LSHIFT => VirtualKey::LShift,
        VK_RSHIFT => VirtualKey::RShift,
        VK_LCONTROL => VirtualKey::LCtrl,
        VK_RCONTROL => VirtualKey::RCtrl,
        VK_LMENU => VirtualKey::LAlt,
        VK_RMENU => VirtualKey::RAlt,
        v @ VK_F1..=VK_F24 => VirtualKey::F((v - VK_F1 + 1) as u8),
        v @ _ => VirtualKey::Other(v as u32),
    }
}

/// Get current key states.
pub fn keyboard_state() -> Vec<VirtualKey> {
    unsafe {
        let mut ks = [0u8; 256];
        GetKeyboardState(ks.as_mut_ptr());
        ks.iter().enumerate().filter_map(|(i, k)| {
            if (k & 0x80) != 0 {
                Some(as_virtual_key(i as i32))
            } else {
                None
            }
        }).collect::<Vec<_>>()
    }
}
