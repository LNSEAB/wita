use crate::context::call_handler;
use crate::device::*;
use crate::last_error;
use crate::EventHandler;
use crate::Window;
use log::debug;
use std::cell::RefCell;
use std::mem::size_of;
use std::ptr::null_mut;
use std::rc::Rc;
use winapi::shared::hidpi::*;
use winapi::shared::hidsdi::*;
use winapi::shared::hidusage::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::fileapi::*;
use winapi::um::handleapi::*;
use winapi::um::winnt::*;
use winapi::um::winuser::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Value {
    I8(i8),
    I16(i16),
    I32(i32),
    U8(u8),
    U16(u16),
    U32(u32),
}

impl Value {
    pub fn signed(&self) -> bool {
        match self {
            Self::I8(_) | Self::I16(_) | Self::I32(_) => true,
            _ => false,
        }
    }

    pub fn unsigned(&self) -> bool {
        !self.signed()
    }

    pub fn bit_size(&self) -> u32 {
        match self {
            Self::I8(_) | Self::U8(_) => 8,
            Self::I16(_) | Self::U16(_) => 16,
            Self::I32(_) | Self::U32(_) => 32,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limit {
    pub min: Value,
    pub max: Value,
}

unsafe fn get_preparsed_data(handle: HANDLE, dest: &mut Vec<u8>) -> Option<()> {
    dest.clear();
    let mut len = 0;
    let ret = GetRawInputDeviceInfoW(handle, RIDI_PREPARSEDDATA, null_mut(), &mut len);
    if (ret as i32) < 0 {
        last_error!("GetRawInputDeviceInfoW");
        return None;
    }
    dest.resize(len as _, 0);
    let ret = GetRawInputDeviceInfoW(handle, RIDI_PREPARSEDDATA, dest.as_mut_ptr() as _, &mut len);
    if (ret as i32) < 0 {
        last_error!("GetRawInputDeviceInfoW");
        return None;
    }
    Some(())
}

unsafe fn get_device_interface(handle: HANDLE) -> Option<Vec<u16>> {
    let mut len = 0;
    let ret = GetRawInputDeviceInfoW(handle, RIDI_DEVICENAME, null_mut(), &mut len);
    if ret != 0 {
        last_error!("GetRawInputDeviceInfoW");
        return None;
    }
    let mut v = vec![0u16; len as usize + 1];
    let ret = GetRawInputDeviceInfoW(handle, RIDI_DEVICENAME, v.as_mut_ptr() as _, &mut len);
    if ret == std::u32::MAX {
        last_error!("GetRawInputDeviceInfoW");
        return None;
    }
    Some(v)
}

unsafe fn get_device_name(interface: &Vec<u16>) -> Option<String> {
    let handle = CreateFileW(
        interface.as_ptr(),
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        null_mut(),
        OPEN_EXISTING,
        0,
        null_mut(),
    );
    if handle == null_mut() {
        last_error!("CreateFileW");
        return None;
    }
    let mut buffer = [0u16; 127];
    let ret = HidD_GetProductString(
        handle,
        buffer.as_mut_ptr() as _,
        (buffer.len() * size_of::<u16>()) as _,
    );
    if ret == 0 {
        CloseHandle(handle);
        return None;
    }
    CloseHandle(handle);
    let end = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..end]))
}

unsafe fn get_raw_input_device_info(handle: HANDLE) -> Option<RID_DEVICE_INFO> {
    let mut len = size_of::<RID_DEVICE_INFO>() as u32;
    let mut info = RID_DEVICE_INFO {
        cbSize: len as _,
        ..Default::default()
    };
    let ret = GetRawInputDeviceInfoW(handle, RIDI_DEVICEINFO, &mut info as *mut _ as _, &mut len);
    if (ret as i32) < 0 {
        last_error!("GetRawInputDeviceInfoW");
        return None;
    }
    Some(info)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeviceType {
    Keyboard,
    Mouse,
    GamePad,
}

#[derive(Clone, Debug)]
pub struct Device {
    handle: HANDLE,
    ty: DeviceType,
    name: Option<String>,
}

impl Device {
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| n.as_str())
    }

    pub fn device_type(&self) -> DeviceType {
        self.ty.clone()
    }

    pub fn raw_handle(&self) -> HANDLE {
        self.handle.clone()
    }
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name().unwrap_or_default())
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Eq for Device {}

#[derive(Clone, Copy, Debug)]
pub struct ValueLimit {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug)]
pub struct KeyboardInfo {
    pub function_num: u32,
    pub indicator_num: u32,
    pub keys_total: u32,
}

#[derive(Debug)]
pub struct MouseInfo {
    pub button_num: u32,
    pub sample_rate: u32,
    pub has_hwheel: bool,
}

#[derive(Default, Debug)]
pub struct GamePadInfo {
    pub button_num: u32,
    pub x: Option<Limit>,
    pub y: Option<Limit>,
    pub z: Option<Limit>,
    pub rx: Option<Limit>,
    pub ry: Option<Limit>,
    pub rz: Option<Limit>,
    pub hat: Option<Limit>,
}

#[derive(Debug)]
pub enum DeviceInfo {
    Keyboard(KeyboardInfo),
    Mouse(MouseInfo),
    GamePad(GamePadInfo),
}

pub fn get_device_info(device: &Device) -> Option<DeviceInfo> {
    unsafe {
        let info = get_raw_input_device_info(device.handle)?;
        match info.dwType {
            RIM_TYPEKEYBOARD => {
                let keyboard = info.u.keyboard();
                Some(DeviceInfo::Keyboard(KeyboardInfo {
                    function_num: keyboard.dwNumberOfFunctionKeys,
                    indicator_num: keyboard.dwNumberOfIndicators,
                    keys_total: keyboard.dwNumberOfKeysTotal,
                }))
            }
            RIM_TYPEMOUSE => {
                let mouse = info.u.mouse();
                Some(DeviceInfo::Mouse(MouseInfo {
                    button_num: mouse.dwNumberOfButtons,
                    sample_rate: mouse.dwSampleRate,
                    has_hwheel: mouse.fHasHorizontalWheel != 0,
                }))
            }
            RIM_TYPEHID => {
                let mut preparsed = vec![];
                get_preparsed_data(device.handle, &mut preparsed)?;
                let p = preparsed.as_mut_ptr() as PHIDP_PREPARSED_DATA;
                let caps = {
                    let mut caps = HIDP_CAPS::default();
                    let ret = HidP_GetCaps(p, &mut caps);
                    if ret != HIDP_STATUS_SUCCESS {
                        return None;
                    }
                    caps
                };
                let button_caps = {
                    let mut caps =
                        vec![HIDP_BUTTON_CAPS::default(); caps.NumberInputButtonCaps as _];
                    let mut len = caps.len() as u16;
                    let ret = HidP_GetButtonCaps(HidP_Input, caps.as_mut_ptr(), &mut len, p);
                    if ret != HIDP_STATUS_SUCCESS {
                        last_error!("HidP_GetButtonCaps");
                        return None;
                    }
                    caps
                };
                let button_num = if button_caps[0].IsRange != 0 {
                    let range = button_caps[0].u.Range();
                    (range.UsageMax - range.UsageMin + 1) as u32
                } else {
                    return None;
                };
                let value_caps = {
                    let mut len = caps.NumberInputValueCaps;
                    let mut caps = vec![HIDP_VALUE_CAPS::default(); len as _];
                    let ret = HidP_GetValueCaps(HidP_Input, caps.as_mut_ptr(), &mut len, p);
                    if ret != HIDP_STATUS_SUCCESS {
                        last_error!("HidP_GetValueCaps");
                        return None;
                    }
                    caps
                };
                let mut info = GamePadInfo {
                    button_num,
                    ..Default::default()
                };
                for i in 0..caps.NumberInputValueCaps as usize {
                    let caps = &value_caps[i];
                    let usage = if caps.IsRange == 0 {
                        caps.u.NotRange().Usage
                    } else {
                        continue;
                    };
                    let limit = if caps.LogicalMin > caps.LogicalMax {
                        match caps.BitSize {
                            b @ _ if b <= 8 => Limit {
                                min: Value::U8(caps.LogicalMin as u8),
                                max: Value::U8(caps.LogicalMax as u8),
                            },
                            b @ _ if b <= 16 => Limit {
                                min: Value::U16(caps.LogicalMin as u16),
                                max: Value::U16(caps.LogicalMax as u16),
                            },
                            b @ _ if b <= 32 => Limit {
                                min: Value::U32(caps.LogicalMin as u32),
                                max: Value::U32(caps.LogicalMax as u32),
                            },
                            _ => return None,
                        }
                    } else {
                        match caps.BitSize {
                            b @ _ if b <= 8 => Limit {
                                min: Value::I8(caps.LogicalMin as i8),
                                max: Value::I8(caps.LogicalMax as i8),
                            },
                            b @ _ if b <= 16 => Limit {
                                min: Value::I16(caps.LogicalMin as i16),
                                max: Value::I16(caps.LogicalMax as i16),
                            },
                            b @ _ if b <= 32 => Limit {
                                min: Value::I32(caps.LogicalMin as i32),
                                max: Value::I32(caps.LogicalMax as i32),
                            },
                            _ => return None,
                        }
                    };
                    match usage {
                        0x30 => info.x = Some(limit),
                        0x31 => info.y = Some(limit),
                        0x32 => info.z = Some(limit),
                        0x33 => info.rx = Some(limit),
                        0x34 => info.ry = Some(limit),
                        0x35 => info.rz = Some(limit),
                        0x39 => info.hat = Some(limit),
                        _ => (),
                    }
                }
                Some(DeviceInfo::GamePad(info))
            }
            _ => unreachable!(),
        }
    }
}

struct GamePadContext {
    device: Device,
    preparsed: Vec<u8>,
    caps: HIDP_CAPS,
    button_caps: Vec<HIDP_BUTTON_CAPS>,
    value_caps: Vec<HIDP_VALUE_CAPS>,
    usage: Vec<USAGE>,
    buttons: Rc<Vec<bool>>,
}

thread_local! {
    static RAW_INPUT_DATA: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
    static GAMEPAD_CONTEXTS: RefCell<Vec<GamePadContext>> = RefCell::new(Vec::new());
    static DEVICE_LIST: RefCell<Vec<Device>> = RefCell::new(Vec::new());
}

unsafe fn register_gamepad_context(device: &Device) {
    GAMEPAD_CONTEXTS.with(|ctxs| {
        let mut ctxs = ctxs.borrow_mut();
        let mut preparsed = vec![];
        if get_preparsed_data(device.raw_handle(), &mut preparsed).is_none() {
            return;
        }
        let p = preparsed.as_mut_ptr() as PHIDP_PREPARSED_DATA;
        let mut caps = HIDP_CAPS::default();
        if HidP_GetCaps(p, &mut caps) != HIDP_STATUS_SUCCESS {
            return;
        }
        let button_caps = {
            let mut len = caps.NumberInputButtonCaps;
            let mut caps = vec![HIDP_BUTTON_CAPS::default(); len as _];
            let ret = HidP_GetButtonCaps(HidP_Input, caps.as_mut_ptr(), &mut len, p);
            if ret != HIDP_STATUS_SUCCESS {
                return;
            }
            caps
        };
        let value_caps = {
            let mut len = caps.NumberInputValueCaps;
            let mut caps = vec![HIDP_VALUE_CAPS::default(); len as _];
            let ret = HidP_GetValueCaps(HidP_Input, caps.as_mut_ptr(), &mut len, p);
            if ret != HIDP_STATUS_SUCCESS {
                return;
            }
            caps
        };
        let button_range = button_caps[0].u.Range();
        let button_num = (button_range.UsageMax - button_range.UsageMin + 1) as usize;
        let usage_num = HidP_MaxUsageListLength(HidP_Input, button_caps[0].UsagePage, p) as usize;
        ctxs.push(GamePadContext {
            device: device.clone(),
            preparsed,
            caps,
            button_caps,
            value_caps,
            usage: vec![USAGE::default(); usage_num],
            buttons: Rc::new(vec![false; button_num]),
        });
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InputOccurence {
    Foreground,
    Background,
}

impl From<WPARAM> for InputOccurence {
    fn from(src: WPARAM) -> InputOccurence {
        match src {
            RIM_INPUT => InputOccurence::Foreground,
            RIM_INPUTSINK => InputOccurence::Background,
            _ => unreachable!(),
        }
    }
}

pub(crate) fn register_devices(wnd: &Window, occurence: InputOccurence) {
    let flags = RIDEV_DEVNOTIFY
        | if occurence == InputOccurence::Background {
            RIDEV_INPUTSINK
        } else {
            0
        };
    let device = [
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_KEYBOARD,
            dwFlags: flags,
            hwndTarget: wnd.raw_handle() as _,
        },
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_MOUSE,
            dwFlags: flags,
            hwndTarget: wnd.raw_handle() as _,
        },
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_JOYSTICK,
            dwFlags: flags,
            hwndTarget: wnd.raw_handle() as _,
        },
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_GAMEPAD,
            dwFlags: flags,
            hwndTarget: wnd.raw_handle() as _,
        },
    ];
    unsafe {
        let ret = RegisterRawInputDevices(
            device.as_ptr(),
            device.len() as _,
            size_of::<RAWINPUTDEVICE>() as _,
        );
        if ret == 0 {
            last_error!("RegisterRawInputDevices");
        }
        let device_list = get_device_list();
        for device in &device_list {
            if get_device_type(device.raw_handle()) == Some(DeviceType::GamePad) {
                register_gamepad_context(device);
            }
        }
        GAMEPAD_CONTEXTS.with(|ctxs| {
            log::debug!("GAMEPAD_CONTEXTS.len = {}", ctxs.borrow().len());
        });
        DEVICE_LIST.with(move |dl| {
            *dl.borrow_mut() = device_list;
        });
    }
}

unsafe fn get_device_type(handle: HANDLE) -> Option<DeviceType> {
    let info = get_raw_input_device_info(handle)?;
    match info.dwType {
        RIM_TYPEKEYBOARD => Some(DeviceType::Keyboard),
        RIM_TYPEMOUSE => Some(DeviceType::Mouse),
        RIM_TYPEHID => {
            let hid = info.u.hid();
            if hid.usUsagePage != HID_USAGE_PAGE_GENERIC {
                return None;
            }
            if hid.usUsage != HID_USAGE_GENERIC_JOYSTICK && hid.usUsage != HID_USAGE_GENERIC_GAMEPAD
            {
                return None;
            }
            Some(DeviceType::GamePad)
        }
        _ => None,
    }
}

pub fn get_device_list() -> Vec<Device> {
    unsafe {
        let mut len = 0;
        let ret = GetRawInputDeviceList(
            std::ptr::null_mut(),
            &mut len,
            size_of::<RAWINPUTDEVICELIST>() as _,
        );
        if (ret as i32) < 0 {
            last_error!("GetRawInputDeviceList");
            return vec![];
        }
        let mut devices = vec![RAWINPUTDEVICELIST::default(); len as usize];
        let ret = GetRawInputDeviceList(
            devices.as_mut_ptr(),
            &mut len,
            size_of::<RAWINPUTDEVICELIST>() as _,
        );
        if (ret as i32) < 0 {
            last_error!("GetRawInputDeviceList");
            return vec![];
        }
        devices
            .iter()
            .filter_map(|device| {
                Some(Device {
                    handle: device.hDevice,
                    ty: get_device_type(device.hDevice)?,
                    name: get_device_interface(device.hDevice).and_then(|i| get_device_name(&i)),
                })
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MousePosition {
    Relative { x: i32, y: i32 },
    Absolute { x: i32, y: i32 },
}

impl MousePosition {
    pub const ABSOLUTE_MAX: i32 = 65535;
}

#[derive(Clone, Copy, Debug)]
pub struct MouseButtonStates(u16);

impl MouseButtonStates {
    pub fn contains(&self, button: MouseButton, state: KeyState) -> bool {
        match state {
            KeyState::Pressed => match button {
                MouseButton::Left => (self.0 & RI_MOUSE_LEFT_BUTTON_DOWN) != 0,
                MouseButton::Right => (self.0 & RI_MOUSE_RIGHT_BUTTON_DOWN) != 0,
                MouseButton::Middle => (self.0 & RI_MOUSE_MIDDLE_BUTTON_DOWN) != 0,
                MouseButton::Ex(0) => (self.0 & RI_MOUSE_BUTTON_4_DOWN) != 0,
                MouseButton::Ex(1) => (self.0 & RI_MOUSE_BUTTON_5_DOWN) != 0,
                _ => false,
            },
            KeyState::Released => match button {
                MouseButton::Left => (self.0 & RI_MOUSE_LEFT_BUTTON_UP) != 0,
                MouseButton::Right => (self.0 & RI_MOUSE_RIGHT_BUTTON_UP) != 0,
                MouseButton::Middle => (self.0 & RI_MOUSE_MIDDLE_BUTTON_UP) != 0,
                MouseButton::Ex(0) => (self.0 & RI_MOUSE_BUTTON_4_UP) != 0,
                MouseButton::Ex(1) => (self.0 & RI_MOUSE_BUTTON_5_UP) != 0,
                _ => false,
            },
        }
    }
}

#[derive(Debug)]
pub struct KeyboardData {
    pub device: Device,
    pub code: KeyCode,
    pub state: KeyState,
    pub extra: u32,
}

#[derive(Debug)]
pub struct MouseData {
    pub device: Device,
    pub position: MousePosition,
    pub wheel: Option<i16>,
    pub hwheel: Option<i16>,
    pub buttons: MouseButtonStates,
    pub extra: u32,
}

#[derive(Debug)]
pub struct GamePadData {
    pub device: Device,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub rx: i32,
    pub ry: i32,
    pub rz: i32,
    pub hat: i32,
    buttons: Rc<Vec<bool>>,
}

impl GamePadData {
    pub fn buttons(&self) -> &Vec<bool> {
        self.buttons.as_ref()
    }
}

#[derive(Debug)]
pub enum InputData {
    Keyboard(KeyboardData),
    Mouse(MouseData),
    GamePad(GamePadData),
}

unsafe fn input_data_keyboard(input: &mut RAWINPUT) -> Option<InputData> {
    let keyboard = input.data.keyboard();
    let code = KeyCode {
        vkey: as_virtual_key(keyboard.VKey as _),
        scan_code: ScanCode(keyboard.MakeCode as _),
    };
    let state = if (keyboard.Flags & RI_KEY_MAKE as u16) != 0 {
        KeyState::Pressed
    } else {
        KeyState::Released
    };
    let handle = input.header.hDevice;
    Some(InputData::Keyboard(KeyboardData {
        device: DEVICE_LIST.with(|dl| {
            dl.borrow()
                .iter()
                .find(|d| d.raw_handle() == handle)
                .cloned()
        })?,
        code,
        state,
        extra: keyboard.ExtraInformation,
    }))
}

unsafe fn input_data_mouse(input: &mut RAWINPUT) -> Option<InputData> {
    let mouse = input.data.mouse();
    let position = if (mouse.usFlags & MOUSE_MOVE_ABSOLUTE) != 0 {
        MousePosition::Absolute { x: 0, y: 0 }
    } else {
        MousePosition::Relative {
            x: mouse.lLastX,
            y: mouse.lLastY,
        }
    };
    let wheel = if (mouse.usButtonFlags & RI_MOUSE_WHEEL) != 0 {
        Some(mouse.usButtonData as i16)
    } else {
        None
    };
    // RI_MOUSE_HWHEEL is missing in winapi-rs
    let hwheel = if (mouse.usButtonFlags & 0x0800) != 0 {
        Some(mouse.usButtonData as i16)
    } else {
        None
    };
    let handle = input.header.hDevice;
    Some(InputData::Mouse(MouseData {
        device: DEVICE_LIST.with(|dl| {
            dl.borrow()
                .iter()
                .find(|d| d.raw_handle() == handle)
                .cloned()
        })?,
        position,
        wheel,
        hwheel,
        buttons: MouseButtonStates(mouse.usButtonFlags),
        extra: mouse.ulExtraInformation,
    }))
}

unsafe fn input_data_gamepad(input: &mut RAWINPUT) -> Option<InputData> {
    GAMEPAD_CONTEXTS.with(|ctxs| {
        let handle = input.header.hDevice;
        let hid = input.data.hid_mut();
        let mut ctxs = ctxs.borrow_mut();
        let ctx = ctxs
            .iter_mut()
            .find(|ctx| ctx.device.raw_handle() == handle)?;
        get_preparsed_data(handle, &mut ctx.preparsed)?;
        let p = ctx.preparsed.as_mut_ptr() as _;
        let mut len = ctx.usage.len() as _;
        let ret = HidP_GetUsages(
            HidP_Input,
            ctx.button_caps[0].UsagePage,
            0,
            ctx.usage.as_mut_ptr(),
            &mut len,
            p,
            hid.bRawData.as_mut_ptr() as _,
            hid.dwSizeHid,
        );
        if ret != HIDP_STATUS_SUCCESS {
            return None;
        }
        {
            let buttons = Rc::get_mut(&mut ctx.buttons).unwrap();
            for btn in buttons.iter_mut() {
                *btn = false;
            }
            for i in 0..(len as usize) {
                buttons[(ctx.usage[i] - ctx.button_caps[0].u.Range().UsageMin) as usize] = true;
            }
        }
        let value_caps = &ctx.value_caps;
        let mut x = 0;
        let mut y = 0;
        let mut z = 0;
        let mut rx = 0;
        let mut ry = 0;
        let mut rz = 0;
        let mut hat = 0;
        for i in 0..ctx.caps.NumberInputValueCaps as usize {
            let mut value = 0;
            let usage = if value_caps[i].IsRange != 0 {
                value_caps[i].u.Range().UsageMin
            } else {
                value_caps[i].u.NotRange().Usage
            };
            let ret = HidP_GetUsageValue(
                HidP_Input,
                value_caps[i].UsagePage,
                0,
                usage,
                &mut value,
                p,
                hid.bRawData.as_mut_ptr() as _,
                hid.dwSizeHid,
            );
            if ret != HIDP_STATUS_SUCCESS {
                continue;
            }
            let value = value as i32;
            if usage == 0x39 {
                hat = value;
            } else {
                match usage {
                    0x30 => x = value,
                    0x31 => y = value,
                    0x32 => z = value,
                    0x33 => rx = value,
                    0x34 => ry = value,
                    0x35 => rz = value,
                    _ => (),
                }
            }
        }
        let handle = input.header.hDevice;
        Some(InputData::GamePad(GamePadData {
            device: DEVICE_LIST.with(|dl| {
                dl.borrow()
                    .iter()
                    .find(|d| d.raw_handle() == handle)
                    .cloned()
            })?,
            x,
            y,
            z,
            rx,
            ry,
            rz,
            hat,
            buttons: ctx.buttons.clone(),
        }))
    })
}

pub(crate) unsafe fn wm_input<T>(
    window: &Window,
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT
where
    T: EventHandler + 'static,
{
    const HEADER_SIZE: u32 = size_of::<RAWINPUTHEADER>() as u32;
    let input_handle = lparam as HRAWINPUT;
    let data = RAW_INPUT_DATA.with(|data| {
        let mut len = 0;
        let ret = GetRawInputData(input_handle, RID_INPUT, null_mut(), &mut len, HEADER_SIZE);
        if (ret as i32) < 0 {
            last_error!("GetRawInputData");
            return None;
        }
        let mut v = data.borrow_mut();
        v.clear();
        v.resize(len as _, 0);
        let ret = GetRawInputData(
            input_handle,
            RID_INPUT,
            v.as_mut_ptr() as _,
            &mut len,
            HEADER_SIZE,
        );
        if (ret as i32) < 0 {
            last_error!("GetRawInputData");
            return None;
        }
        Some(data.clone())
    });
    if data.is_none() {
        return DefWindowProcW(hwnd, WM_INPUT, wparam, lparam);
    }
    let data = data.unwrap();
    call_handler(move |eh: &mut T, _| {
        let input = &mut *(data.borrow_mut().as_mut_ptr() as *mut RAWINPUT);
        let data = match input.header.dwType {
            RIM_TYPEKEYBOARD => input_data_keyboard(input),
            RIM_TYPEMOUSE => input_data_mouse(input),
            RIM_TYPEHID => input_data_gamepad(input),
            _ => unreachable!(),
        };
        if let Some(data) = data {
            eh.raw_input(window, &data);
        }
    });
    DefWindowProcW(hwnd, WM_INPUT, wparam, lparam)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeviceChangeState {
    Arrival,
    Removal,
}

pub(crate) unsafe fn wm_input_device_change<T>(
    window: &Window,
    _: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT
where
    T: EventHandler + 'static,
{
    let handle = lparam as HANDLE;
    match wparam as u32 {
        GIDC_ARRIVAL => {
            let ty = get_device_type(handle);
            let name = get_device_interface(handle).and_then(|i| get_device_name(&i));
            if ty.is_none() || name.is_none() {
                return 0;
            }
            let device = Device {
                handle,
                ty: ty.unwrap(),
                name: name,
            };
            if device.ty == DeviceType::GamePad {
                register_gamepad_context(&device);
            }
            DEVICE_LIST.with(|dl| dl.borrow_mut().push(device.clone()));
            call_handler(|eh: &mut T, _| {
                eh.raw_input_device_change(window, &device, DeviceChangeState::Arrival);
            });
            debug!("device arrival: {:?} {:?}", handle, device.name);
        }
        GIDC_REMOVAL => {
            if let Some(device) = DEVICE_LIST.with(|dl| {
                dl.borrow()
                    .iter()
                    .find(|d| d.raw_handle() == handle)
                    .cloned()
            }) {
                call_handler(|eh: &mut T, _| {
                    eh.raw_input_device_change(window, &device, DeviceChangeState::Removal);
                });
                debug!("device removal: {:?} {:?}", handle, device.name);
            }
            GAMEPAD_CONTEXTS.with(|ctxs| {
                let mut ctxs = ctxs.borrow_mut();
                let index = ctxs.iter().position(|ctx| ctx.device.handle == handle);
                if let Some(index) = index {
                    ctxs.remove(index);
                }
            });
            DEVICE_LIST.with(|dl| {
                let mut dl = dl.borrow_mut();
                let index = dl.iter().position(|d| d.raw_handle() == handle);
                if let Some(index) = index {
                    dl.remove(index);
                }
            });
        }
        _ => unreachable!(),
    }
    0
}
