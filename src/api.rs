mod imm;

use crate::geometry::*;
use winapi::shared::{minwindef::*, windef::*};
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

pub fn adjust_window_rect(size: PhysicalSize<f32>, style: DWORD, ex_style: DWORD, dpi: u32) -> RECT {
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
