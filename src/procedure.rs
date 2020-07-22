use crate::{api::*, context::*, device::*, geometry::*, ime, window::Window};
use std::panic::catch_unwind;
use winapi::shared::{minwindef::*, windef::*, windowsx::*};
use winapi::um::winuser::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub(crate) enum UserMessage {
    SetPosition,
    SetInnerSize,
    EnableIme,
    DisableIme,
}

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

fn as_virtual_key(wparam: WPARAM) -> VirtualKey {
    const ZERO: i32 = b'0' as i32;
    const Z: i32 = b'Z' as i32;
    let value = wparam as i32;
    match value {
        v @ ZERO..=Z => VirtualKey::Char((v as u8).into()),
        VK_ESCAPE => VirtualKey::Esc,
        VK_TAB => VirtualKey::Tab,
        VK_CAPITAL => VirtualKey::CapsLock,
        VK_SHIFT => VirtualKey::Shift,
        VK_CONTROL => VirtualKey::Ctrl,
        VK_MENU => VirtualKey::Alt,
        VK_BACK => VirtualKey::BackSpace,
        VK_RETURN => VirtualKey::Enter,
        VK_SPACE => VirtualKey::Space,
        VK_SNAPSHOT => VirtualKey::PrintScreen,
        VK_SCROLL => VirtualKey::ScrollLock,
        VK_PAUSE => VirtualKey::Pause,
        VK_INSERT => VirtualKey::Insert,
        VK_DELETE => VirtualKey::Delete,
        VK_HOME => VirtualKey::Home,
        VK_END => VirtualKey::End,
        VK_PRIOR => VirtualKey::PageUp,
        VK_NEXT => VirtualKey::PageDown,
        VK_UP => VirtualKey::Up,
        VK_DOWN => VirtualKey::Down,
        VK_LEFT => VirtualKey::Left,
        VK_RIGHT => VirtualKey::Right,
        VK_NUMLOCK => VirtualKey::NumLock,
        v @ VK_NUMPAD0..=VK_NUMPAD9 => VirtualKey::NumPad((v - VK_NUMPAD0) as u8),
        VK_ADD => VirtualKey::NumAdd,
        VK_SUBTRACT => VirtualKey::NumSub,
        VK_MULTIPLY => VirtualKey::NumMul,
        VK_DIVIDE => VirtualKey::NumDiv,
        VK_DECIMAL => VirtualKey::NumDecimal,
        v @ VK_F1..=VK_F24 => VirtualKey::F((v - VK_F1 + 1) as u8),
        v @ _ => VirtualKey::Other(v as u32),
    }
}

fn key_input(window: &Window, state: KeyState, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let scan_code = ScanCode(((lparam >> 16) & 0x7f) as u32);
    call_handler(|eh, _| {
        eh.key_input(window, KeyCode::new(as_virtual_key(wparam), scan_code), state);
    });
    0
}

pub(crate) unsafe extern "system" fn window_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ret = catch_unwind(|| {
        let window = find_window(hwnd);
        if window.is_none() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        let window = window.unwrap();
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(hwnd, &mut ps);
                call_handler(|eh, _| {
                    eh.draw(&window);
                });
                EndPaint(hwnd, &ps);
                0
            }
            WM_MOUSEMOVE => {
                call_handler(|eh, state| {
                    let position = lparam_to_point(lparam).to_logical(window.scale_factor());
                    update_buttons(&mut state.mouse_buttons, wparam);
                    if state.entered_window.is_none() {
                        TrackMouseEvent(&mut TRACKMOUSEEVENT {
                            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as DWORD,
                            dwFlags: TME_LEAVE,
                            hwndTrack: hwnd,
                            dwHoverTime: 0,
                        });
                        state.entered_window = Some(window.clone());
                        eh.cursor_entered(
                            &window,
                            MouseState {
                                position,
                                buttons: &state.mouse_buttons,
                            },
                        );
                    } else {
                        eh.cursor_moved(
                            &window,
                            MouseState {
                                position,
                                buttons: &state.mouse_buttons,
                            },
                        );
                    }
                });
                0
            }
            WM_MOUSELEAVE => {
                call_handler(|eh, state| {
                    state.entered_window = None;
                    update_buttons(&mut state.mouse_buttons, wparam);
                    let mut pos = POINT::default();
                    GetCursorPos(&mut pos);
                    eh.cursor_leaved(
                        &window,
                        MouseState {
                            position: PhysicalPosition::new(pos.x as f32, pos.y as f32)
                                .to_logical(window.scale_factor()),
                            buttons: &mut state.mouse_buttons,
                        },
                    );
                });
                0
            }
            WM_LBUTTONDOWN => mouse_input(&window, MouseButton::Left, KeyState::Pressed, wparam, lparam),
            WM_RBUTTONDOWN => mouse_input(&window, MouseButton::Right, KeyState::Pressed, wparam, lparam),
            WM_MBUTTONDOWN => mouse_input(&window, MouseButton::Middle, KeyState::Pressed, wparam, lparam),
            WM_XBUTTONDOWN => mouse_input(&window, wparam_to_button(wparam), KeyState::Pressed, wparam, lparam),
            WM_LBUTTONUP => mouse_input(&window, MouseButton::Left, KeyState::Released, wparam, lparam),
            WM_RBUTTONUP => mouse_input(&window, MouseButton::Right, KeyState::Released, wparam, lparam),
            WM_MBUTTONUP => mouse_input(&window, MouseButton::Middle, KeyState::Released, wparam, lparam),
            WM_XBUTTONUP => mouse_input(&window, wparam_to_button(wparam), KeyState::Released, wparam, lparam),
            WM_KEYDOWN => key_input(&window, KeyState::Pressed, wparam, lparam),
            WM_KEYUP => key_input(&window, KeyState::Released, wparam, lparam),
            WM_CHAR => {
                call_handler(|eh, _| {
                    std::char::from_u32(wparam as u32).map(|c| eh.char_input(&window, c));
                });
                0
            }
            WM_IME_SETCONTEXT => {
                let lparam = {
                    let state = window.state.read().unwrap();
                    let mut lparam = lparam;
                    if !state.visible_ime_composition_window {
                        lparam &= !ISC_SHOWUICOMPOSITIONWINDOW;
                    }
                    if !state.visible_ime_candidate_window {
                        lparam &= !ISC_SHOWUICANDIDATEWINDOW;
                        lparam &= !(ISC_SHOWUICANDIDATEWINDOW << 1);
                        lparam &= !(ISC_SHOWUICANDIDATEWINDOW << 2);
                        lparam &= !(ISC_SHOWUICANDIDATEWINDOW << 3);
                    }
                    lparam
                };
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_IME_STARTCOMPOSITION => {
                call_handler(|eh, _| {
                    let imc = Imc::get(hwnd);
                    let state = window.state.read().unwrap();
                    if state.visible_ime_composition_window {
                        imc.set_composition_window_position(state.ime_position);
                    }
                    if state.visible_ime_candidate_window {
                        imc.set_candidate_window_position(state.ime_position, state.visible_ime_composition_window);
                    }
                    eh.ime_start_composition(&window);
                });
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_IME_COMPOSITION => {
                call_handler(|eh, _| {
                    let imc = Imc::get(hwnd);
                    if lparam & GCS_COMPSTR as LPARAM != 0 {
                        if let Some(CompositionString::CompStr(s)) = imc.get_composition_string(GCS_COMPSTR) {
                            if let Some(CompositionString::CompAttr(attrs)) = imc.get_composition_string(GCS_COMPATTR) {
                                eh.ime_composition(
                                    &window,
                                    &ime::Composition::new(s, attrs),
                                    imc.get_candidate_list().as_ref(),
                                );
                            }
                        }
                    }
                    if lparam & GCS_RESULTSTR as LPARAM != 0 {
                        if let Some(CompositionString::ResultStr(s)) = imc.get_composition_string(GCS_RESULTSTR) {
                            if let Some(CompositionString::CompAttr(attrs)) = imc.get_composition_string(GCS_COMPATTR) {
                                eh.ime_composition(&window, &ime::Composition::new(s, attrs), None);
                            }
                        }
                    }
                });
                let show_composition_window = {
                    let state = window.state.read().unwrap();
                    state.visible_ime_composition_window
                };
                if show_composition_window {
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                } else {
                    0
                }
            }
            WM_IME_ENDCOMPOSITION => {
                call_handler(|eh, _| {
                    let imc = Imc::get(hwnd);
                    let ret = imc.get_composition_string(GCS_RESULTSTR);
                    let ret = if let Some(CompositionString::ResultStr(s)) = &ret {
                        Some(s.as_str())
                    } else {
                        None
                    };
                    eh.ime_end_composition(&window, ret);
                });
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_ACTIVATE => {
                if (wparam & WA_ACTIVE as WPARAM) != 0 || (wparam & WA_CLICKACTIVE as WPARAM) != 0 {
                    call_handler(|eh, _| eh.activated(&window));
                } else {
                    call_handler(|eh, _| eh.inactivated(&window));
                }
                0
            }
            WM_SIZE => {
                let value = lparam as DWORD;
                let size = PhysicalSize::new(LOWORD(value) as f32, HIWORD(value) as f32);
                call_handler(|eh, _| eh.resized(&window, size));
                0
            }
            WM_WINDOWPOSCHANGED => {
                let pos = &*(lparam as *const WINDOWPOS);
                if pos.flags & SWP_NOMOVE == 0 {
                    call_handler(|eh, _| eh.moved(&window, ScreenPosition::new(pos.x, pos.y)));
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
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
                if root_window().map_or(true, |wnd| wnd.raw_handle() == hwnd as *const std::ffi::c_void) {
                    PostQuitMessage(0);
                }
                0
            }
            WM_NCCREATE => {
                EnableNonClientDpiScaling(hwnd);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_USER => {
                match wparam {
                    w if w == UserMessage::SetPosition as usize => {
                        let state = window.state.read().unwrap();
                        SetWindowPos(hwnd, std::ptr::null_mut(), state.set_position.x, state.set_position.y, 0, 0, SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE);
                    }
                    w if w == UserMessage::SetInnerSize as usize => {
                        let state = window.state.read().unwrap();
                        SetWindowPos(hwnd, std::ptr::null_mut(), 0, 0, state.set_inner_size.width as i32, state.set_inner_size.height as i32, SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE);
                    }
                    w if w == UserMessage::EnableIme as usize => {
                        let state = window.state.read().unwrap();
                        state.ime_context.enable();
                    }
                    w if w == UserMessage::DisableIme as usize => {
                        let state = window.state.read().unwrap();
                        state.ime_context.disable();
                    }
                    _ => unreachable!(),
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    });
    ret.unwrap_or_else(|e| {
        set_unwind(e);
        0
    })
}
