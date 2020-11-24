mod imm;

use crate::geometry::*;
use std::sync::Once;
use winapi::shared::{minwindef::*, windef::*, winerror::S_OK};
use winapi::um::{imm::*, shellscalingapi::*, winnt::*, winuser::*};

pub use imm::*;

pub fn get_dpi_from_point(pt: ScreenPosition) -> u32 {
    unsafe {
        let mut dpi_x = 0;
        let mut _dpi_y = 0;
        GetDpiForMonitor(
            MonitorFromPoint(POINT { x: pt.x, y: pt.y }, MONITOR_DEFAULTTOPRIMARY),
            MDT_DEFAULT,
            &mut dpi_x,
            &mut _dpi_y,
        );
        dpi_x
    }
}

pub fn adjust_window_rect(
    size: PhysicalSize<f32>,
    style: DWORD,
    ex_style: DWORD,
    dpi: u32,
) -> RECT {
    unsafe {
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: size.width as i32,
            bottom: size.height as i32,
        };
        AdjustWindowRectExForDpi(&mut rc, style, FALSE, ex_style, dpi);
        rc
    }
}

pub fn enable_dpi_awareness() {
    static ENABLE_DPI_AWARENESS: Once = Once::new();
    unsafe {
        ENABLE_DPI_AWARENESS.call_once(|| {
            if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) == TRUE {
                return;
            } else if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE) == TRUE
            {
                return;
            } else if SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE) == S_OK {
                return;
            }
        });
    }
}

pub fn enable_gui_thread() {
    unsafe {
        IsGUIThread(TRUE);
    }
}
