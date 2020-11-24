use crate::geometry::*;
use winapi::shared::{minwindef::*, windef::*};
use winapi::um::winuser::*;

/// Describes monitor info.
#[derive(Clone, Debug)]
pub struct Monitor {
    hmonitor: HMONITOR,
    pub position: ScreenPosition,
    pub size: PhysicalSize<f32>,
    pub is_primary: bool,
}

impl PartialEq for Monitor {
    fn eq(&self, other: &Monitor) -> bool {
        self.hmonitor == other.hmonitor
    }
}

unsafe extern "system" fn get_monitors_proc(
    hmonitor: HMONITOR,
    _: HDC,
    rc: LPRECT,
    lparam: LPARAM,
) -> BOOL {
    let v = &mut *(lparam as *mut Vec<Monitor>);
    let rc = &*rc;
    let mut info = MONITORINFO::default();
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    GetMonitorInfoW(hmonitor, &mut info);
    v.push(Monitor {
        hmonitor,
        position: ScreenPosition::new(rc.left, rc.top),
        size: PhysicalSize::new((rc.right - rc.left) as f32, (rc.bottom - rc.top) as f32),
        is_primary: (info.dwFlags & MONITORINFOF_PRIMARY) != 0,
    });
    TRUE
}

/// Return monitors info.
pub fn get_monitors() -> Vec<Monitor> {
    unsafe {
        let len = GetSystemMetrics(SM_CMONITORS) as usize;
        let mut v = Vec::with_capacity(len);
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            Some(get_monitors_proc),
            (&mut v) as *mut Vec<Monitor> as LPARAM,
        );
        v
    }
}

/// A screen position to a monitor.
pub fn monitor_from_point(point: ScreenPosition) -> Option<Monitor> {
    unsafe {
        let hmonitor = MonitorFromPoint(
            POINT {
                x: point.x,
                y: point.y,
            },
            MONITOR_DEFAULTTONULL,
        );
        if hmonitor == std::ptr::null_mut() {
            return None;
        }
        let mut info = MONITORINFO::default();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        GetMonitorInfoW(hmonitor, &mut info);
        Some(Monitor {
            hmonitor,
            position: ScreenPosition::new(info.rcMonitor.left, info.rcMonitor.top),
            size: PhysicalSize::new(
                (info.rcMonitor.right - info.rcMonitor.left) as f32,
                (info.rcMonitor.bottom - info.rcMonitor.top) as f32,
            ),
            is_primary: (info.dwFlags & MONITORINFOF_PRIMARY) != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn monitors_len() {
        let monitors = get_monitors();
        unsafe {
            assert_eq!(monitors.len(), GetSystemMetrics(SM_CMONITORS) as usize);
        }
    }
}
