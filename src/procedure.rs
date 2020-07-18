use crate::{api::*, context::*, device::*, geometry::*, window::Window};
use winapi::shared::{minwindef::*, windef::*, windowsx::*};
use winapi::um::winuser::*;

fn lparam_to_point(lparam: LPARAM) -> PhysicalPosition<f32> {
    PhysicalPosition::new(GET_X_LPARAM(lparam) as f32, GET_Y_LPARAM(lparam) as f32)
}

fn wparam_to_button(wparam: WPARAM) -> MouseButton {
    match GET_XBUTTON_WPARAM(wparam) {
        XBUTTON1 => MouseButton::Ex(0),
        XBUTTON2 => MouseButton::Ex(1),
        _ => unreachable!(),
    }
}

fn update_buttons(buttons: &mut Vec<MouseButton>, wparam: WPARAM) {
    buttons.clear();
    let values = GET_KEYSTATE_WPARAM(wparam) as usize;
    if values & MK_LBUTTON != 0 {
        buttons.push(MouseButton::Left);
    }
    if values & MK_RBUTTON != 0 {
        buttons.push(MouseButton::Right);
    }
    if values & MK_MBUTTON != 0 {
        buttons.push(MouseButton::Middle);
    }
    if values & MK_XBUTTON1 != 0 {
        buttons.push(MouseButton::Ex(0));
    }
    if values & MK_XBUTTON2 != 0 {
        buttons.push(MouseButton::Ex(1));
    }
}

unsafe fn mouse_input(
    window: &Window,
    button: MouseButton,
    button_state: KeyState,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    call_handler(|eh, state| {
        update_buttons(&mut state.mouse_buttons, wparam);
        eh.mouse_input(
            window,
            button,
            button_state,
            MouseState {
                position: lparam_to_point(lparam).to_logical(window.scale_factor()),
                buttons: &state.mouse_buttons,
            },
        );
    });
    0
}

pub(crate) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let window = find_window(hwnd);
    if window.is_none() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let window = window.unwrap();
    match msg {
        WM_LBUTTONDOWN => mouse_input(
            &window,
            MouseButton::Left,
            KeyState::Pressed,
            wparam,
            lparam,
        ),
        WM_RBUTTONDOWN => mouse_input(
            &window,
            MouseButton::Right,
            KeyState::Pressed,
            wparam,
            lparam,
        ),
        WM_MBUTTONDOWN => mouse_input(
            &window,
            MouseButton::Middle,
            KeyState::Pressed,
            wparam,
            lparam,
        ),
        WM_XBUTTONDOWN => mouse_input(
            &window,
            wparam_to_button(wparam),
            KeyState::Pressed,
            wparam,
            lparam,
        ),
        WM_LBUTTONUP => mouse_input(
            &window,
            MouseButton::Left,
            KeyState::Released,
            wparam,
            lparam,
        ),
        WM_RBUTTONUP => mouse_input(
            &window,
            MouseButton::Right,
            KeyState::Released,
            wparam,
            lparam,
        ),
        WM_MBUTTONUP => mouse_input(
            &window,
            MouseButton::Middle,
            KeyState::Released,
            wparam,
            lparam,
        ),
        WM_XBUTTONUP => mouse_input(
            &window,
            wparam_to_button(wparam),
            KeyState::Released,
            wparam,
            lparam,
        ),
        WM_ACTIVATE => {
            if (wparam & WA_ACTIVE as WPARAM) != 0 || (wparam & WA_CLICKACTIVE as WPARAM) != 0 {
                call_handler(|eh, _| eh.activated(&window));
            } else {
                call_handler(|eh, _| eh.inactivated(&window));
            }
            0
        }
        WM_DPICHANGED => {
            let rc = *(lparam as *const RECT);
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                rc.left,
                rc.top,
                rc.right - rc.left,
                rc.bottom - rc.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            call_handler(|eh, _| eh.dpi_changed(&window));
            0
        }
        WM_GETDPISCALEDSIZE => {
            let prev_dpi = GetDpiForWindow(hwnd) as i32;
            let next_dpi = wparam as i32;
            let mut rc = RECT::default();
            GetClientRect(hwnd, &mut rc);
            let size = PhysicalSize::new(
                ((rc.right - rc.left) * next_dpi / prev_dpi) as f32,
                ((rc.bottom - rc.top) * next_dpi / prev_dpi) as f32,
            );
            let rc = adjust_window_rect(
                size,
                GetWindowLongPtrW(hwnd, GWL_STYLE) as DWORD,
                GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as DWORD,
                next_dpi as u32,
            );
            let mut ret = (lparam as *mut SIZE).as_mut().unwrap();
            ret.cx = rc.right - rc.left;
            ret.cy = rc.bottom - rc.top;
            TRUE as LRESULT
        }
        WM_DESTROY => {
            call_handler(|eh, _| eh.closed(&window));
            if root_window().map_or(true, |wnd| {
                wnd.raw_handle() == hwnd as *const std::ffi::c_void
            }) {
                PostQuitMessage(0);
            }
            0
        }
        WM_NCCREATE => {
            EnableNonClientDpiScaling(hwnd);
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
