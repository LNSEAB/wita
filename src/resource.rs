use crate::bindings::Windows::Win32::{
    UI::Controls::*, UI::MenusAndResources::*, System::SystemServices::*, UI::WindowsAndMessaging::*,
};
use std::path::{Path, PathBuf};

#[inline]
fn make_int_resource(id: u16) -> PWSTR {
    PWSTR(id as _)
}

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
            Icon::Resource(id) => LoadImageW(
                hinst,
                make_int_resource(*id),
                IMAGE_ICON,
                cx,
                cy,
                LR_SHARED,
            ),
            Icon::File(path) => {
                LoadImageW(
                    HINSTANCE::NULL,
                    path.to_string_lossy().as_ref(),
                    IMAGE_ICON,
                    cx,
                    cy,
                    LR_SHARED
                        | LR_LOADFROMFILE,
                )
            }
        }
    };
    if icon == HANDLE::NULL {
        panic!("cannot load the icon");
    }
    HICON(icon.0)
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
