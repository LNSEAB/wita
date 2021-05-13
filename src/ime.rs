//! An IME composition string and a candidate list

use crate::bindings::Windows::Win32::{DisplayDevices::*, Intl::*, WindowsAndMessaging::*};
use crate::geometry::*;

/// Describes composition character attributes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Attribute {
    Input,
    TargetConverted,
    Converted,
    TargetNotConverted,
    Error,
    FixedConverted,
}

/// A composition character and a composition attribute.
#[derive(Debug)]
pub struct CompositionChar {
    pub ch: char,
    pub attr: Attribute,
}

/// A composition string.
#[derive(Debug)]
pub struct Composition {
    chars: Vec<CompositionChar>,
}

impl Composition {
    pub(crate) fn new(s: String, attrs: Vec<Attribute>) -> Self {
        Self {
            chars: s
                .chars()
                .zip(attrs.into_iter())
                .map(|(ch, attr)| CompositionChar { ch, attr })
                .collect::<Vec<_>>(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn iter(&self) -> impl Iterator + '_ {
        self.chars.iter()
    }
}

impl std::iter::IntoIterator for Composition {
    type Item = CompositionChar;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.chars.into_iter()
    }
}

impl std::ops::Index<usize> for Composition {
    type Output = CompositionChar;
    fn index(&self, index: usize) -> &Self::Output {
        &self.chars[index]
    }
}

/// A candidate list.
#[derive(Debug)]
pub struct CandidateList {
    list: Vec<String>,
    selection: usize,
}

impl CandidateList {
    pub(crate) fn new(list: Vec<String>, selection: usize) -> Self {
        Self { list, selection }
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn iter(&self) -> impl Iterator + '_ {
        self.list.iter()
    }

    pub fn selection(&self) -> (usize, &str) {
        (self.selection, &self.list[self.selection])
    }
}

impl std::ops::Index<usize> for CandidateList {
    type Output = str;
    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
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
            ImmAssociateContextEx(self.hwnd, HIMC::NULL, IACE_IGNORENOCONTEXT);
        }
    }
}

impl Drop for ImmContext {
    fn drop(&mut self) {
        unsafe {
            ImmAssociateContextEx(self.hwnd, HIMC::NULL, IACE_DEFAULT);
            ImmDestroyContext(self.himc);
        }
    }
}

pub enum CompositionString {
    CompStr(String),
    CompAttr(Vec<Attribute>),
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

    pub fn get_composition_string(&self, index: u32) -> Option<CompositionString> {
        unsafe fn get_string(himc: HIMC, index: u32) -> Option<String> {
            let byte_len = ImmGetCompositionStringW(himc, index, std::ptr::null_mut(), 0);
            if byte_len == IMM_ERROR_NODATA || byte_len == IMM_ERROR_GENERAL {
                return None;
            }
            let len = byte_len as usize / std::mem::size_of::<u16>();
            let mut buf = Vec::with_capacity(len);
            buf.set_len(len);
            ImmGetCompositionStringW(himc, index, buf.as_mut_ptr() as *mut _, byte_len as u32);
            let s = String::from_utf16_lossy(&buf);
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }

        unsafe fn get_attrs(himc: HIMC) -> Option<Vec<Attribute>> {
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
                byte_len as u32,
            );
            Some(
                buf.into_iter()
                    .map(|v| match v as u32 {
                        ATTR_INPUT => Attribute::Input,
                        ATTR_TARGET_CONVERTED => Attribute::TargetConverted,
                        ATTR_CONVERTED => Attribute::Converted,
                        ATTR_TARGET_NOTCONVERTED => Attribute::TargetNotConverted,
                        ATTR_INPUT_ERROR => Attribute::Error,
                        ATTR_FIXEDCONVERTED => Attribute::FixedConverted,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>(),
            )
        }

        unsafe {
            match index {
                GCS_COMPSTR => get_string(self.himc, GCS_COMPSTR).map(CompositionString::CompStr),
                GCS_COMPATTR => get_attrs(self.himc).map(CompositionString::CompAttr),
                GCS_RESULTSTR => {
                    get_string(self.himc, GCS_RESULTSTR).map(CompositionString::ResultStr)
                }
                _ => None,
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
            let ret = ImmGetCandidateListW(self.himc, 0, buf.as_mut_ptr() as *mut _, size as u32);
            if ret == 0 {
                return None;
            }
            let obj = &*(buf.as_ptr() as *const CANDIDATELIST);
            let mut list: Vec<String> = Vec::with_capacity(obj.dwCount as usize);
            for i in 0..(obj.dwCount as usize) {
                let offset =
                    std::slice::from_raw_parts(&obj.dwOffset as *const u32, obj.dwCount as usize);
                let p = buf.as_ptr().offset(offset[i] as isize) as *const u16;
                let len = (0..isize::MAX).position(|i| *p.offset(i) == 0).unwrap();
                let slice = std::slice::from_raw_parts(p, len);
                list.push(String::from_utf16_lossy(slice));
            }
            Some(CandidateList::new(list, obj.dwSelection as usize))
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
