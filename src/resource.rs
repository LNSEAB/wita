use crate::bindings::Windows::Win32::{
    Controls::*, MenusAndResources::*, SystemServices::*, WindowsAndMessaging::*,
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
                GDI_IMAGE_TYPE::IMAGE_ICON,
                cx,
                cy,
                IMAGE_FLAGS::LR_SHARED,
            ),
            Icon::File(path) => {
                LoadImageW(
                    HINSTANCE::NULL,
                    path.to_string_lossy().as_ref(),
                    GDI_IMAGE_TYPE::IMAGE_ICON,
                    cx,
                    cy,
                    IMAGE_FLAGS(
                        IMAGE_FLAGS::LR_SHARED.0
                            | IMAGE_FLAGS::LR_LOADFROMFILE.0,
                    ),
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
            GetSystemMetrics(SYSTEM_METRICS_INDEX::SM_CXICON),
            GetSystemMetrics(SYSTEM_METRICS_INDEX::SM_CYICON),
        )
    }
}

pub(crate) fn load_small_icon(icon: &Icon, hinst: HINSTANCE) -> HICON {
    unsafe {
        load_icon_impl(
            hinst,
            icon,
            GetSystemMetrics(SYSTEM_METRICS_INDEX::SM_CXSMICON),
            GetSystemMetrics(SYSTEM_METRICS_INDEX::SM_CYSMICON),
        )
    }
}
