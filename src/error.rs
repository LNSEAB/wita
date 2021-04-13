use crate::bindings::Windows::Win32::{Debug::*, SystemServices::*};
use std::ptr::{null, null_mut};

#[doc(hidden)]
#[macro_export]
macro_rules! last_error {
    ($s:literal) => {
        ::log::error!(
            "{}({}) {}: 0x{:<08x}",
            file!(),
            line!(),
            $s,
            $crate::bindings::Windows::Win32::Debug::GetLastError()
        )
    };
}

fn format_message(code: u32) -> Option<String> {
    unsafe {
        let mut p = null_mut() as *mut u16;
        let len = FormatMessageW(
            FORMAT_MESSAGE_OPTIONS(
                FORMAT_MESSAGE_OPTIONS::FORMAT_MESSAGE_ALLOCATE_BUFFER.0
                    | FORMAT_MESSAGE_OPTIONS::FORMAT_MESSAGE_FROM_SYSTEM.0
                    | FORMAT_MESSAGE_OPTIONS::FORMAT_MESSAGE_IGNORE_INSERTS.0,
            ),
            null(),
            code,
            0,
            std::mem::transmute(&mut p),
            0,
            null_mut(),
        );
        if len == 0 {
            return None;
        }
        let buffer = std::slice::from_raw_parts(p, len as usize);
        let ret = String::from_utf16_lossy(buffer);
        LocalFree(p as _);
        Some(ret)
    }
}

/// Represents an Win32 API error.
#[derive(Default, Debug)]
pub struct ApiError(u32);

impl ApiError {
    pub fn new() -> Self {
        unsafe { Self(GetLastError()) }
    }

    pub fn code(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", format_message(self.0).unwrap_or_default())
    }
}

impl std::error::Error for ApiError {}
