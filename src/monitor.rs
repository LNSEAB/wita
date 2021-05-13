use crate::bindings::Windows::Win32::{
    DisplayDevices::*, Gdi::*, SystemServices::*, WindowsAndMessaging::*,
};
use crate::geometry::*;

/// Describes monitor info.
#[derive(Clone, Debug)]
pub struct Monitor {
    hmonitor: HMONITOR,
    pub position: ScreenPosition,
    pub size: PhysicalSize<u32>,
    pub is_primary: bool,
}

impl PartialEq for Monitor {
    fn eq(&self, other: &Monitor) -> bool {
        self.hmonitor == other.hmonitor
    }
}

extern "system" fn get_monitors_proc(
    hmonitor: HMONITOR,
    _: HDC,
    rc: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    unsafe {
        let v = &mut *(lparam.0 as *mut Vec<Monitor>);
        let rc = &*rc;
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        GetMonitorInfoW(hmonitor, &mut info);
        v.push(Monitor {
            hmonitor,
            position: ScreenPosition::new(rc.left, rc.top),
            size: PhysicalSize::new((rc.right - rc.left) as u32, (rc.bottom - rc.top) as u32),
            is_primary: (info.dwFlags & MONITORINFOF_PRIMARY) != 0,
        });
        TRUE
    }
}

/// Return monitors info.
pub fn get_monitors() -> Vec<Monitor> {
    unsafe {
        let len = GetSystemMetrics(SYSTEM_METRICS_INDEX::SM_CMONITORS) as usize;
        let mut v = Vec::with_capacity(len);
        EnumDisplayMonitors(
            HDC::NULL,
            std::ptr::null_mut(),
            Some(get_monitors_proc),
            LPARAM((&mut v) as *mut Vec<Monitor> as _),
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
            MONITOR_FROM_FLAGS::MONITOR_DEFAULTTONULL,
        );
        if hmonitor == HMONITOR::NULL {
            return None;
        }
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        GetMonitorInfoW(hmonitor, &mut info);
        Some(Monitor {
            hmonitor,
            position: ScreenPosition::new(info.rcMonitor.left, info.rcMonitor.top),
            size: PhysicalSize::new(
                (info.rcMonitor.right - info.rcMonitor.left) as u32,
                (info.rcMonitor.bottom - info.rcMonitor.top) as u32,
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
            assert_eq!(
                monitors.len(),
                GetSystemMetrics(SYSTEM_METRICS_INDEX::SM_CMONITORS) as usize
            );
        }
    }
}
