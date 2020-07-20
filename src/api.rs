use crate::geometry::*;
use winapi::shared::{minwindef::*, windef::*};
use winapi::um::{shellscalingapi::*, winuser::*, imm::*, winnt::*};

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

pub const GCS_COMPSTR: DWORD = 0x0008;
pub const GCS_CURSORPOS: DWORD = 0x0080;
pub const GCS_RESULTSTR: DWORD = 0x0800;
pub const ISC_SHOWUICOMPOSITIONWINDOW: LPARAM = 0x80000000;
pub const ISC_SHOWUICANDIDATEWINDOW: LPARAM = 0x00000001;
pub const IMN_OPENCANDIDATE: WPARAM = 0x0005;
pub const IMR_COMPOSITIONWINDOW: WPARAM = 0x0001;
pub const IMR_CANDIDATEWINDOW: WPARAM = 0x0002;
pub const IMM_ERROR_NODATA: LONG = -1;
pub const IMM_ERROR_GENERAL: LONG = -2;
pub const IACE_CHILDREN: DWORD = 0x0001;
pub const IACE_DEFAULT: DWORD = 0x0010;
pub const IACE_IGNORENOCONTEXT: DWORD = 0x0020;

#[repr(C)]
#[allow(non_snake_case)]
struct CANDIDATEFORM {
    dwIndex: DWORD,
    dwStyle: DWORD,
    ptCurrentPos: POINT,
    rcArea: RECT,
}

#[repr(C)]
#[allow(non_snake_case)]
struct CANDIDATELIST {
    dwSize: DWORD,
    dwStyle: DWORD,
    dwCount: DWORD,
    dwSelection: DWORD,
    dwPageStart: DWORD,
    dwPageSize: DWORD,
    dwOffset: [DWORD; 1],
}

extern "system" {
    fn ImmCreateContext() -> HIMC;
    fn ImmDestroyContext(himc: HIMC);
    fn ImmSetCandidateWindow(himc: HIMC, form: *mut CANDIDATEFORM) -> BOOL;
    fn ImmGetCompositionStringW(himc: HIMC, index: DWORD, lpbuf: LPVOID, buflen: DWORD) -> LONG;
    fn ImmGetCandidateListW(himc: HIMC, index: DWORD, list: *mut CANDIDATELIST, len: DWORD) -> DWORD;
    fn ImmAssociateContextEx(hwnd: HWND, himc: HIMC, flag: DWORD) -> BOOL;
}

pub struct CandidateList {
    pub selection: usize,
    pub page_size: usize,
    pub list: Vec<String>,
}

pub struct ImmContext {
    hwnd: HWND,
    himc: HIMC,
}

impl ImmContext {
    pub fn new(hwnd: HWND) -> Self {
        unsafe {
            let himc = ImmCreateContext();
            ImmAssociateContextEx(hwnd, himc, IACE_CHILDREN);
            Self {
                hwnd,
                himc,
            }
        }
    }
}

impl Drop for ImmContext {
    fn drop(&mut self) {
        unsafe {
            ImmAssociateContextEx(self.hwnd, std::ptr::null_mut(), IACE_DEFAULT);
            ImmDestroyContext(self.himc);
        }
    }
}

pub struct Imc {
    hwnd: HWND,
    himc: HIMC,
}

impl Imc {
    pub fn get(hwnd: HWND) -> Self {
        let himc = unsafe { ImmGetContext(hwnd) };
        Self { hwnd, himc }
    }

    pub fn set_composition_window_position(&self, position: PhysicalPosition<i32>) {
        unsafe {
            let pt = POINT {
                x: position.x,
                y: position.y,
            };
            let mut form = COMPOSITIONFORM {
                dwStyle: CFS_POINT,
                ptCurrentPos: pt,
                rcArea: RECT::default(),
            };
            ImmSetCompositionWindow(self.himc, &mut form);
        }
    }

    pub fn set_candidate_window_position(&self, position: PhysicalPosition<i32>, enable_exclude_rect: bool) {
        unsafe {
            let pt = POINT {
                x: position.x,
                y: position.y,
            };
            let mut form = CANDIDATEFORM {
                dwStyle: CFS_CANDIDATEPOS,
                dwIndex: 0,
                ptCurrentPos: pt,
                rcArea: RECT::default(),
            };
            ImmSetCandidateWindow(self.himc, &mut form);
            if !enable_exclude_rect {
                let mut form = CANDIDATEFORM {
                    dwStyle: CFS_EXCLUDE,
                    dwIndex: 0,
                    ptCurrentPos: pt,
                    rcArea: RECT {
                        left: pt.x,
                        top: pt.y,
                        right: pt.x,
                        bottom: pt.y,
                    },
                };
                ImmSetCandidateWindow(self.himc, &mut form);
            }
        }
    }

    pub fn get_composition_string(&self, index: DWORD) -> Option<String> {
        unsafe {
            let byte_len = ImmGetCompositionStringW(self.himc, index, std::ptr::null_mut(), 0);
            if byte_len == IMM_ERROR_NODATA || byte_len == IMM_ERROR_GENERAL {
                return None;
            }
            let len = byte_len as usize / std::mem::size_of::<u16>();
            let mut buf = Vec::with_capacity(len);
            buf.set_len(len);
            ImmGetCompositionStringW(self.himc, index, buf.as_mut_ptr() as *mut _, byte_len as DWORD);
            let s = String::from_utf16_lossy(&buf);
            if s == "" {
                None
            } else {
                Some(s)
            }
        }
    }

    pub fn get_candidate_list(&self) -> Option<CandidateList> {
        unsafe {
            let size = ImmGetCandidateListW(self.himc, 0, std::ptr::null_mut(), 0) as usize;
            if size == 0 {
                return None;
            }
            let mut buf: Vec<u8> = Vec::with_capacity(size);
            buf.set_len(size);
            let ret = ImmGetCandidateListW(self.himc, 0, buf.as_mut_ptr() as *mut _, size as DWORD);
            if ret == 0 {
                return None;
            }
            let obj = &*(buf.as_ptr() as *const CANDIDATELIST);
            let begin_ptr = buf.as_ptr().offset(std::mem::size_of::<CANDIDATELIST>() as isize);
            let mut list: Vec<String> = Vec::with_capacity(obj.dwCount as usize);
            for i in 0..(obj.dwCount as usize) {
                let p = begin_ptr.offset(obj.dwOffset[i] as isize) as *const u16;
                let len = (0..isize::MAX).position(|i| *p.offset(i) == 0).unwrap();
                let slice = std::slice::from_raw_parts(p, len);
                list.push(String::from_utf16_lossy(slice));
            }
            Some(CandidateList {
                selection: obj.dwSelection as usize,
                page_size: obj.dwPageSize as usize,
                list,
            })
        }
    }
}

impl Drop for Imc {
    fn drop(&mut self) {
        unsafe {
            ImmReleaseContext(self.hwnd, self.himc);
        }
    }
}
