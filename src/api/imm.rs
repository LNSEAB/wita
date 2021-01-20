use super::*;
use crate::ime;

pub const GCS_COMPSTR: DWORD = 0x0008;
pub const GCS_COMPATTR: DWORD = 0x0010;
pub const GCS_RESULTSTR: DWORD = 0x0800;
pub const ISC_SHOWUICOMPOSITIONWINDOW: LPARAM = 0x80000000;
pub const ISC_SHOWUICANDIDATEWINDOW: LPARAM = 0x00000001;
pub const IMM_ERROR_NODATA: LONG = -1;
pub const IMM_ERROR_GENERAL: LONG = -2;
pub const IACE_CHILDREN: DWORD = 0x0001;
pub const IACE_DEFAULT: DWORD = 0x0010;
pub const IACE_IGNORENOCONTEXT: DWORD = 0x0020;
pub const ATTR_INPUT: u8 = 0x00;
pub const ATTR_TARGET_CONVERTED: u8 = 0x01;
pub const ATTR_CONVERTED: u8 = 0x02;
pub const ATTR_TARGET_NOTCONVERTED: u8 = 0x03;
pub const ATTR_INPUT_ERROR: u8 = 0x04;
pub const ATTR_FIXEDCONVERTED: u8 = 0x5;

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
#[derive(Debug)]
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
    fn ImmGetCandidateListW(
        himc: HIMC,
        index: DWORD,
        list: *mut CANDIDATELIST,
        len: DWORD,
    ) -> DWORD;
    fn ImmAssociateContextEx(hwnd: HWND, himc: HIMC, flag: DWORD) -> BOOL;
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
            Self { hwnd, himc }
        }
    }

    pub fn enable(&self) {
        unsafe {
            ImmAssociateContextEx(self.hwnd, self.himc, IACE_CHILDREN);
        }
    }

    pub fn disable(&self) {
        unsafe {
            ImmAssociateContextEx(self.hwnd, std::ptr::null_mut(), IACE_IGNORENOCONTEXT);
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

pub enum CompositionString {
    CompStr(String),
    CompAttr(Vec<ime::Attribute>),
    ResultStr(String),
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

    pub fn set_candidate_window_position(
        &self,
        position: PhysicalPosition<i32>,
        enable_exclude_rect: bool,
    ) {
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

    pub fn get_composition_string(&self, index: DWORD) -> Option<CompositionString> {
        unsafe fn get_string(himc: HIMC, index: DWORD) -> Option<String> {
            let byte_len = ImmGetCompositionStringW(himc, index, std::ptr::null_mut(), 0);
            if byte_len == IMM_ERROR_NODATA || byte_len == IMM_ERROR_GENERAL {
                return None;
            }
            let len = byte_len as usize / std::mem::size_of::<u16>();
            let mut buf = Vec::with_capacity(len);
            buf.set_len(len);
            ImmGetCompositionStringW(himc, index, buf.as_mut_ptr() as *mut _, byte_len as DWORD);
            let s = String::from_utf16_lossy(&buf);
            if s == "" {
                None
            } else {
                Some(s)
            }
        }

        unsafe fn get_attrs(himc: HIMC) -> Option<Vec<ime::Attribute>> {
            let byte_len = ImmGetCompositionStringW(himc, GCS_COMPATTR, std::ptr::null_mut(), 0);
            if byte_len == IMM_ERROR_NODATA || byte_len == IMM_ERROR_GENERAL {
                return None;
            }
            let len = byte_len as usize;
            let mut buf: Vec<u8> = Vec::with_capacity(len);
            buf.set_len(len);
            ImmGetCompositionStringW(
                himc,
                GCS_COMPATTR,
                buf.as_mut_ptr() as *mut _,
                byte_len as DWORD,
            );
            Some(
                buf.into_iter()
                    .map(|v| match v {
                        ATTR_INPUT => ime::Attribute::Input,
                        ATTR_TARGET_CONVERTED => ime::Attribute::TargetConverted,
                        ATTR_CONVERTED => ime::Attribute::Converted,
                        ATTR_TARGET_NOTCONVERTED => ime::Attribute::TargetNotConverted,
                        ATTR_INPUT_ERROR => ime::Attribute::Error,
                        ATTR_FIXEDCONVERTED => ime::Attribute::FixedConverted,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>(),
            )
        }

        unsafe {
            match index {
                GCS_COMPSTR => {
                    get_string(self.himc, GCS_COMPSTR).map(|s| CompositionString::CompStr(s))
                }
                GCS_COMPATTR => get_attrs(self.himc).map(|v| CompositionString::CompAttr(v)),
                GCS_RESULTSTR => {
                    get_string(self.himc, GCS_RESULTSTR).map(|s| CompositionString::ResultStr(s))
                }
                _ => None,
            }
        }
    }

    pub fn get_candidate_list(&self) -> Option<ime::CandidateList> {
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
            let mut list: Vec<String> = Vec::with_capacity(obj.dwCount as usize);
            for i in 0..(obj.dwCount as usize) {
                let offset =
                    std::slice::from_raw_parts(&obj.dwOffset as *const DWORD, obj.dwCount as usize);
                let p = buf.as_ptr().offset(offset[i] as isize) as *const u16;
                let len = (0..isize::MAX).position(|i| *p.offset(i) == 0).unwrap();
                let slice = std::slice::from_raw_parts(p, len);
                list.push(String::from_utf16_lossy(slice));
            }
            Some(ime::CandidateList::new(list, obj.dwSelection as usize))
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
