use std::path::{Path, PathBuf};
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::winuser::*;

/// Describes a icon.
#[derive(Clone, Debug)]
pub enum Icon {
    /// A icon from a resource id.
    Resource(u16),
    /// A icon from a file.
    File(PathBuf),
}

impl Icon {
    pub fn from_path(path: impl AsRef<Path>) -> Icon {
        Icon::File(path.as_ref().to_path_buf())
    }
}

fn load_icon_impl(hinst: HINSTANCE, icon: &Icon, cx: i32, cy: i32) -> HICON {
    let icon = unsafe {
        match icon {
            Icon::Resource(id) => {
                LoadImageW(hinst, MAKEINTRESOURCEW(*id), IMAGE_ICON, cx, cy, LR_SHARED)
            }
            Icon::File(path) => {
                let wpath = path
                    .to_string_lossy()
                    .encode_utf16()
                    .chain(Some(0))
                    .collect::<Vec<_>>();
                LoadImageW(
                    std::ptr::null_mut(),
                    wpath.as_ptr(),
                    IMAGE_ICON,
                    cx,
                    cy,
                    LR_SHARED | LR_LOADFROMFILE,
                )
            }
        }
    };
    if icon == std::ptr::null_mut() {
        panic!("cannot load the icon");
    }
    icon as _
}

pub(crate) fn load_icon(icon: &Icon, hinst: HINSTANCE) -> HICON {
    unsafe {
        load_icon_impl(
            hinst,
            icon,
            GetSystemMetrics(SM_CXICON),
            GetSystemMetrics(SM_CYICON),
        )
    }
}

pub(crate) fn load_small_icon(icon: &Icon, hinst: HINSTANCE) -> HICON {
    unsafe {
        load_icon_impl(
            hinst,
            icon,
            GetSystemMetrics(SM_CXSMICON),
            GetSystemMetrics(SM_CYSMICON),
        )
    }
}
