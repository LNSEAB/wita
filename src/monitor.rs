use crate::bindings::windows::win32::{
    display_devices::*, gdi::*, system_services::*, windows_and_messaging::*,
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
            cb_size: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        GetMonitorInfoW(hmonitor, &mut info);
        v.push(Monitor {
            hmonitor,
            position: ScreenPosition::new(rc.left, rc.top),
            size: PhysicalSize::new((rc.right - rc.left) as u32, (rc.bottom - rc.top) as u32),
            is_primary: (info.dw_flags & MONITORINFOF_PRIMARY) != 0,
        });
        BOOL(1)
    }
}

/// Return monitors info.
pub fn get_monitors() -> Vec<Monitor> {
    unsafe {
        let len = GetSystemMetrics(GetSystemMetrics_nIndexFlags::SM_CMONITORS) as usize;
        let mut v = Vec::with_capacity(len);
        EnumDisplayMonitors(
            HDC(0),
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
            MonitorFrom_dwFlags::MONITOR_DEFAULTTONULL,
        );
        if hmonitor == HMONITOR(0) {
            return None;
        }
        let mut info = MONITORINFO {
            cb_size: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        GetMonitorInfoW(hmonitor, &mut info);
        Some(Monitor {
            hmonitor,
            position: ScreenPosition::new(info.rc_monitor.left, info.rc_monitor.top),
            size: PhysicalSize::new(
                (info.rc_monitor.right - info.rc_monitor.left) as u32,
                (info.rc_monitor.bottom - info.rc_monitor.top) as u32,
            ),
            is_primary: (info.dw_flags & MONITORINFOF_PRIMARY) != 0,
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
                GetSystemMetrics(GetSystemMetrics_nIndexFlags::SM_CMONITORS) as usize
            );
        }
    }
}
