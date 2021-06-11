use crate::bindings::Windows::Win32::{
    Foundation::*, UI::KeyboardAndMouseInput::*, UI::WindowsAndMessaging::*,
};
use crate::geometry::*;
#[cfg(feature = "serde")]
use serde::{de::*, *};

/// Describes the state of a keyboard key and a mouse button.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum KeyState {
    Pressed,
    Released,
}

/// Describes mouse buttons.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
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

impl std::fmt::Display for VirtualKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Char(c) => write!(f, "{}", c.to_ascii_uppercase()),
            Self::NumPad(n) => write!(f, "NumPad{}", n),
            Self::F(n) => write!(f, "F{}", n),
            Self::Other(n) => write!(f, "Other{}", n),
            k => write!(f, "{:?}", k),
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for VirtualKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

#[cfg(feature = "serde")]
struct VirtualKeyVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for VirtualKeyVisitor {
    type Value = VirtualKey;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid value")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "Esc" => Ok(VirtualKey::Esc),
            "Tab" => Ok(VirtualKey::Tab),
            "CapsLock" => Ok(VirtualKey::CapsLock),
            "Shift" => Ok(VirtualKey::Shift),
            "Ctrl" => Ok(VirtualKey::Ctrl),
            "Alt" => Ok(VirtualKey::Alt),
            "BackSpace" => Ok(VirtualKey::BackSpace),
            "Enter" => Ok(VirtualKey::Enter),
            "Space" => Ok(VirtualKey::Space),
            "PrintScreen" => Ok(VirtualKey::PrintScreen),
            "ScrollLock" => Ok(VirtualKey::ScrollLock),
            "Pause" => Ok(VirtualKey::Pause),
            "Insert" => Ok(VirtualKey::Insert),
            "Delete" => Ok(VirtualKey::Delete),
            "Home" => Ok(VirtualKey::Home),
            "End" => Ok(VirtualKey::End),
            "PageUp" => Ok(VirtualKey::PageUp),
            "PageDown" => Ok(VirtualKey::PageDown),
            "Up" => Ok(VirtualKey::Up),
            "Down" => Ok(VirtualKey::Down),
            "Left" => Ok(VirtualKey::Left),
            "Right" => Ok(VirtualKey::Right),
            "NumLock" => Ok(VirtualKey::NumLock),
            "NumAdd" => Ok(VirtualKey::NumAdd),
            "NumSub" => Ok(VirtualKey::NumSub),
            "NumMul" => Ok(VirtualKey::NumMul),
            "NumDiv" => Ok(VirtualKey::NumDiv),
            "NumDecimal" => Ok(VirtualKey::NumDecimal),
            "LShift" => Ok(VirtualKey::LShift),
            "RShift" => Ok(VirtualKey::RShift),
            "LCtrl" => Ok(VirtualKey::LCtrl),
            "RCtrl" => Ok(VirtualKey::RCtrl),
            "LAlt" => Ok(VirtualKey::LAlt),
            "RAlt" => Ok(VirtualKey::RAlt),
            _ if v.len() == 1 => {
                let c = v.chars().next().unwrap();
                if !c.is_ascii_control() {
                    Ok(VirtualKey::Char(c.to_ascii_uppercase()))
                } else {
                    Err(serde::de::Error::custom("invalid value"))
                }
            }
            _ if v.starts_with("NumPad") => Ok(VirtualKey::NumPad(
                v.trim_matches(|c| !char::is_numeric(c))
                    .parse()
                    .map_err(|_| serde::de::Error::custom("invalid value"))?,
            )),
            _ if v.starts_with('F') => Ok(VirtualKey::F(
                v.trim_matches(|c| !char::is_numeric(c))
                    .parse()
                    .map_err(|_| serde::de::Error::custom("invalid value"))?,
            )),
            _ if v.starts_with("Other") => Ok(VirtualKey::Other(
                v.trim_matches(|c| !char::is_numeric(c))
                    .parse()
                    .map_err(|_| serde::de::Error::custom("invalid value"))?,
            )),
            _ => Err(serde::de::Error::custom("invalid value")),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for VirtualKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(VirtualKeyVisitor)
    }
}

/// A keyboard scan code
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

pub fn as_virtual_key(k: u32) -> VirtualKey {
    const ZERO: u32 = b'0' as _;
    const Z: u32 = b'Z' as _;
    match k {
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
        VK_OEM_MINUS => VirtualKey::Char('-'),
        VK_OEM_PLUS => VirtualKey::Char(';'),
        VK_OEM_COMMA => VirtualKey::Char(','),
        VK_OEM_PERIOD => VirtualKey::Char('.'),
        VK_OEM_1 => VirtualKey::Char(':'),
        VK_OEM_2 => VirtualKey::Char('/'),
        VK_OEM_3 => VirtualKey::Char('@'),
        VK_OEM_4 => VirtualKey::Char('['),
        VK_OEM_5 => VirtualKey::Char('\\'),
        VK_OEM_6 => VirtualKey::Char(']'),
        VK_OEM_7 => VirtualKey::Char('^'),
        VK_OEM_102 => VirtualKey::Char('_'),
        v @ VK_F1..=VK_F24 => VirtualKey::F((v - VK_F1 + 1) as u8),
        v => VirtualKey::Other(v as u32),
    }
}

pub fn to_raw_virtual_key(k: VirtualKey) -> u32 {
    const ZERO: char = '0' as _;
    const Z: char = 'Z' as _;
    match k {
        VirtualKey::Char(c) if (ZERO..=Z).contains(&c) => c as u32,
        VirtualKey::Esc => VK_ESCAPE,
        VirtualKey::Tab => VK_TAB,
        VirtualKey::CapsLock => VK_CAPITAL,
        VirtualKey::Shift => VK_SHIFT,
        VirtualKey::Ctrl => VK_CONTROL,
        VirtualKey::Alt => VK_MENU,
        VirtualKey::BackSpace => VK_BACK,
        VirtualKey::Enter => VK_RETURN,
        VirtualKey::Space => VK_SPACE,
        VirtualKey::PrintScreen => VK_SNAPSHOT,
        VirtualKey::ScrollLock => VK_SCROLL,
        VirtualKey::Pause => VK_PAUSE,
        VirtualKey::Insert => VK_INSERT,
        VirtualKey::Delete => VK_DELETE,
        VirtualKey::Home => VK_HOME,
        VirtualKey::End => VK_END,
        VirtualKey::PageUp => VK_PRIOR,
        VirtualKey::PageDown => VK_NEXT,
        VirtualKey::Up => VK_UP,
        VirtualKey::Down => VK_DOWN,
        VirtualKey::Left => VK_LEFT,
        VirtualKey::Right => VK_RIGHT,
        VirtualKey::NumLock => VK_NUMLOCK,
        VirtualKey::NumPad(n) => VK_NUMPAD0 + n as u32,
        VirtualKey::NumAdd => VK_ADD,
        VirtualKey::NumSub => VK_SUBTRACT,
        VirtualKey::NumMul => VK_MULTIPLY,
        VirtualKey::NumDiv => VK_DIVIDE,
        VirtualKey::NumDecimal => VK_DECIMAL,
        VirtualKey::LShift => VK_LSHIFT,
        VirtualKey::RShift => VK_RSHIFT,
        VirtualKey::LCtrl => VK_LCONTROL,
        VirtualKey::RCtrl => VK_RCONTROL,
        VirtualKey::LAlt => VK_LMENU,
        VirtualKey::RAlt => VK_RMENU,
        VirtualKey::Char('-') => VK_OEM_MINUS,
        VirtualKey::Char(';') => VK_OEM_PLUS,
        VirtualKey::Char(',') => VK_OEM_COMMA,
        VirtualKey::Char('.') => VK_OEM_PERIOD,
        VirtualKey::Char(':') => VK_OEM_1,
        VirtualKey::Char('/') => VK_OEM_2,
        VirtualKey::Char('@') => VK_OEM_3,
        VirtualKey::Char('[') => VK_OEM_4,
        VirtualKey::Char('\\') => VK_OEM_5,
        VirtualKey::Char(']') => VK_OEM_6,
        VirtualKey::Char('^') => VK_OEM_7,
        VirtualKey::Char('_') => VK_OEM_102,
        VirtualKey::F(n) => VK_F1 + n as u32 - 1,
        VirtualKey::Other(x) => x,
        _ => unreachable!(),
    }
}

pub fn get_key_state(k: VirtualKey) -> bool {
    unsafe { GetKeyState(to_raw_virtual_key(k) as _) & 0x80 != 0 }
}

/// Get current key states.
pub fn keyboard_state(keys: &mut Vec<VirtualKey>) {
    let mut buffer = [0u8; 256];
    unsafe {
        GetKeyboardState(buffer.as_mut_ptr());
    }
    keys.clear();
    for (i, k) in buffer.iter().enumerate() {
        if (k & 0x80) != 0 {
            keys.push(as_virtual_key(i as u32));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn serde_test() {
        fn se_de(k: VirtualKey) -> serde_json::Result<bool> {
            let se = serde_json::to_string(&k)?;
            Ok(serde_json::from_str::<VirtualKey>(&se)? == k)
        }

        assert!(se_de(VirtualKey::Char('A')).unwrap());
        assert!(se_de(VirtualKey::NumPad(0)).unwrap());
        assert!(se_de(VirtualKey::F(1)).unwrap());
        assert!(se_de(VirtualKey::Other(230)).unwrap());
        let vks = [
            VirtualKey::Esc,
            VirtualKey::Tab,
            VirtualKey::CapsLock,
            VirtualKey::Shift,
            VirtualKey::Ctrl,
            VirtualKey::Alt,
            VirtualKey::BackSpace,
            VirtualKey::Enter,
            VirtualKey::Space,
            VirtualKey::PrintScreen,
            VirtualKey::ScrollLock,
            VirtualKey::Pause,
            VirtualKey::Insert,
            VirtualKey::Delete,
            VirtualKey::Home,
            VirtualKey::End,
            VirtualKey::PageUp,
            VirtualKey::PageDown,
            VirtualKey::Up,
            VirtualKey::Down,
            VirtualKey::Left,
            VirtualKey::Right,
            VirtualKey::NumLock,
            VirtualKey::NumAdd,
            VirtualKey::NumSub,
            VirtualKey::NumMul,
            VirtualKey::NumDiv,
            VirtualKey::NumDecimal,
            VirtualKey::LShift,
            VirtualKey::RShift,
            VirtualKey::LCtrl,
            VirtualKey::RCtrl,
            VirtualKey::LAlt,
            VirtualKey::RAlt,
        ];
        for k in &vks {
            assert!(se_de(*k).unwrap());
        }
    }

    #[test]
    fn convert_virtual_key() {
        for c in '0'..='Z' {
            assert!(as_virtual_key(to_raw_virtual_key(VirtualKey::Char(c))) == VirtualKey::Char(c));
        }
        for i in 0..=9 {
            assert!(
                as_virtual_key(to_raw_virtual_key(VirtualKey::NumPad(i))) == VirtualKey::NumPad(i)
            );
        }
        for i in 1..=24 {
            assert!(as_virtual_key(to_raw_virtual_key(VirtualKey::F(i))) == VirtualKey::F(i));
        }
        let vks = [
            VirtualKey::Esc,
            VirtualKey::Tab,
            VirtualKey::CapsLock,
            VirtualKey::Shift,
            VirtualKey::Ctrl,
            VirtualKey::Alt,
            VirtualKey::BackSpace,
            VirtualKey::Enter,
            VirtualKey::Space,
            VirtualKey::PrintScreen,
            VirtualKey::ScrollLock,
            VirtualKey::Pause,
            VirtualKey::Insert,
            VirtualKey::Delete,
            VirtualKey::Home,
            VirtualKey::End,
            VirtualKey::PageUp,
            VirtualKey::PageDown,
            VirtualKey::Up,
            VirtualKey::Down,
            VirtualKey::Left,
            VirtualKey::Right,
            VirtualKey::NumLock,
            VirtualKey::NumAdd,
            VirtualKey::NumSub,
            VirtualKey::NumMul,
            VirtualKey::NumDiv,
            VirtualKey::NumDecimal,
            VirtualKey::LShift,
            VirtualKey::RShift,
            VirtualKey::LCtrl,
            VirtualKey::RCtrl,
            VirtualKey::LAlt,
            VirtualKey::RAlt,
        ];
        for &k in &vks {
            assert!(as_virtual_key(to_raw_virtual_key(k)) == k);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cursor {
    AppStarting,
    Arrow,
    Cross,
    Hand,
    Help,
    IBeam,
    No,
    SizeAll,
    SizeNESW,
    SizeNS,
    SizeNWSE,
    SizeWE,
    SizeUpArrow,
    Wait,
}

impl Cursor {
    pub(crate) fn name(&self) -> PWSTR {
        match self {
            Self::AppStarting => IDC_APPSTARTING,
            Self::Arrow => IDC_ARROW,
            Self::Cross => IDC_CROSS,
            Self::Hand => IDC_HAND,
            Self::Help => IDC_HELP,
            Self::IBeam => IDC_IBEAM,
            Self::No => IDC_NO,
            Self::SizeAll => IDC_SIZEALL,
            Self::SizeNESW => IDC_SIZENESW,
            Self::SizeNS => IDC_SIZENS,
            Self::SizeNWSE => IDC_SIZENWSE,
            Self::SizeWE => IDC_SIZEWE,
            Self::SizeUpArrow => IDC_UPARROW,
            Self::Wait => IDC_WAIT,
        }
    }

    #[inline]
    pub(crate) fn set(&self) {
        unsafe {
            SetCursor(LoadCursorW(HINSTANCE::NULL, self.name()));
        }
    }
}

impl Default for Cursor {
    #[inline]
    fn default() -> Self {
        Self::Arrow
    }
}
