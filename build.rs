fn main() {
    windows::build!(
        Windows::Win32::WindowsAndMessaging::*,
        Windows::Win32::HiDpi::*,
        Windows::Win32::Gdi::{
            MonitorFromPoint,
            GetMonitorInfoW,
            EnumDisplayMonitors,
            BeginPaint,
            EndPaint,
            GetStockObject,
            RedrawWindow,
        },
        Windows::Win32::KeyboardAndMouseInput::*,
        Windows::Win32::SystemServices::{
            BOOL,
            HINSTANCE,
            LocalFree,
            GetModuleHandleW,
        },
        Windows::Win32::DisplayDevices::{
            POINT,
            RECT,
        },
        Windows::Win32::Debug::{
            FormatMessageW,
            GetLastError,
            FORMAT_MESSAGE_OPTIONS,
        },
        Windows::Win32::Intl::*,
        Windows::Win32::Shell::{
            DragAcceptFiles,
            DragQueryFileW,
            DragQueryPoint,
            DragFinish,
        },
        Windows::Win32::MenusAndResources::{
            HICON,
        },
        Windows::Win32::Controls::{
            WM_MOUSELEAVE,
        },
        Windows::Win32::WindowsProgramming::{
            CloseHandle,
        },
        Windows::Win32::FileSystem::{
            CreateFileW,
        },
        Windows::Win32::Hid::*,
    );
}
