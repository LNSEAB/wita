use crate::bindings::windows::win32::{
    display_devices::*, gdi::*, hi_dpi::*, system_services::*, windows_and_messaging::*,
};
use crate::geometry::*;
use std::sync::Once;

pub fn get_dpi_from_point(pt: ScreenPosition) -> u32 {
    unsafe {
        let mut dpi_x = 0;
        let mut _dpi_y = 0;
        GetDpiForMonitor(
            MonitorFromPoint(
                POINT { x: pt.x, y: pt.y },
                MonitorFrom_dwFlags::MONITOR_DEFAULTTOPRIMARY,
            ),
            MONITOR_DPI_TYPE::MDT_DEFAULT,
            &mut dpi_x,
            &mut _dpi_y,
        )
        .ok()
        .ok();
        dpi_x
    }
}

pub fn adjust_window_rect(size: PhysicalSize<u32>, style: u32, ex_style: u32, dpi: u32) -> RECT {
    unsafe {
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: size.width as i32,
            bottom: size.height as i32,
        };
        AdjustWindowRectExForDpi(&mut rc, style, BOOL(0), ex_style, dpi);
        rc
    }
}

pub fn enable_dpi_awareness() {
    static ENABLE_DPI_AWARENESS: Once = Once::new();
    unsafe {
        ENABLE_DPI_AWARENESS.call_once(|| {
            if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT(-4)) != BOOL(1)
                && SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT(-3)) != BOOL(1)
            {
                SetProcessDpiAwareness(PROCESS_DPI_AWARENESS::PROCESS_PER_MONITOR_DPI_AWARE)
                    .ok()
                    .ok();
            }
        });
    }
}

pub fn enable_gui_thread() {
    unsafe {
        IsGUIThread(BOOL(1));
    }
}
