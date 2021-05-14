use crate::bindings::Windows::Win32::{
    Globalization::*, Graphics::Gdi::*, System::SystemServices::*, UI::Controls::*,
    UI::DisplayDevices::*, UI::HiDpi::*, UI::KeyboardAndMouseInput::*, UI::Shell::*,
    UI::WindowsAndMessaging::*,
};
#[cfg(feature = "raw_input")]
use crate::raw_input;
use crate::{api::*, context::*, device::*, event::EventHandler, geometry::*, ime, window::Window};
use std::panic::catch_unwind;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub(crate) enum UserMessage {
    SetTitle,
    SetPosition,
    SetInnerSize,
    EnableIme,
    DisableIme,
    SetStyle,
    AcceptDragFiles,
}

#[inline]
fn loword(x: i32) -> i16 {
    (x & 0xffff) as _
}

#[inline]
fn hiword(x: i32) -> i16 {
    ((x >> 16) & 0xffff) as _
}

#[inline]
fn get_x_lparam(lp: LPARAM) -> i16 {
    (lp.0 & 0xffff) as _
}

#[inline]
fn get_y_lparam(lp: LPARAM) -> i16 {
    ((lp.0 >> 16) & 0xffff) as _
}

#[inline]
fn get_xbutton_wparam(wp: WPARAM) -> u16 {
    ((wp.0 >> 16) & 0xffff) as _
}

#[inline]
fn get_keystate_wparam(wp: WPARAM) -> u32 {
    (wp.0 & 0xffff) as _
}

#[inline]
fn lparam_to_point(lparam: LPARAM) -> PhysicalPosition<i32> {
    PhysicalPosition::new(get_x_lparam(lparam) as i32, get_y_lparam(lparam) as i32)
}

#[inline]
fn wparam_to_button(wparam: WPARAM) -> MouseButton {
    match get_xbutton_wparam(wparam) {
        0x0001 => MouseButton::Ex(0),
        0x0002 => MouseButton::Ex(1),
        _ => unreachable!(),
    }
}

fn update_buttons(buttons: &mut Vec<MouseButton>, wparam: WPARAM) {
    buttons.clear();
    let values = get_keystate_wparam(wparam);
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

unsafe fn mouse_input<T: EventHandler + 'static>(
    window: &Window,
    button: MouseButton,
    button_state: KeyState,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    call_handler(|eh: &mut T, state| {
        let mouse_buttons = &mut state.mouse_buttons;
        update_buttons(mouse_buttons, wparam);
        eh.mouse_input(
            window,
            button,
            button_state,
            MouseState {
                position: lparam_to_point(lparam),
                buttons: mouse_buttons,
            },
        );
    });
    LRESULT(0)
}

fn key_input<T: EventHandler + 'static>(
    window: &Window,
    state: KeyState,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let scan_code = ScanCode(((lparam.0 >> 16) & 0x7f) as u32);
    call_handler(|eh: &mut T, _| {
        eh.key_input(
            window,
            KeyCode::new(as_virtual_key(wparam.0 as i32), scan_code),
            state,
            (lparam.0 >> 30) & 0x01 != 0,
        );
    });
    LRESULT(0)
}

pub(crate) extern "system" fn window_proc<T: EventHandler + 'static>(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ret = catch_unwind(|| unsafe {
        let window = find_window(hwnd);
        if window.is_none() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        let window = window.unwrap();
        let handle = &window.handle;
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(hwnd, &mut ps);
                call_handler(|eh: &mut T, _| eh.draw(handle));
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            #[cfg(feature = "raw_input")]
            WM_INPUT => raw_input::wm_input::<T>(handle, hwnd, wparam, lparam),
            WM_MOUSEMOVE => {
                call_handler(|eh: &mut T, state| {
                    let position = lparam_to_point(lparam);
                    update_buttons(&mut state.mouse_buttons, wparam);
                    if state.entered_window.is_none() {
                        TrackMouseEvent(&mut TRACKMOUSEEVENT {
                            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as _,
                            dwFlags: TME_LEAVE,
                            hwndTrack: hwnd,
                            dwHoverTime: 0,
                        });
                        state.entered_window = Some(window.clone());
                        eh.cursor_entered(
                            handle,
                            MouseState {
                                position,
                                buttons: &state.mouse_buttons,
                            },
                        );
                    } else {
                        eh.cursor_moved(
                            handle,
                            MouseState {
                                position,
                                buttons: &state.mouse_buttons,
                            },
                        );
                    }
                });
                LRESULT(0)
            }
            WM_MOUSELEAVE => {
                call_handler(|eh: &mut T, state| {
                    state.entered_window = None;
                    update_buttons(&mut state.mouse_buttons, wparam);
                    let mut pos = POINT::default();
                    GetCursorPos(&mut pos);
                    eh.cursor_leaved(
                        handle,
                        MouseState {
                            position: PhysicalPosition::new(pos.x, pos.y),
                            buttons: &mut state.mouse_buttons,
                        },
                    );
                });
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                mouse_input::<T>(handle, MouseButton::Left, KeyState::Pressed, wparam, lparam)
            }
            WM_RBUTTONDOWN => mouse_input::<T>(
                handle,
                MouseButton::Right,
                KeyState::Pressed,
                wparam,
                lparam,
            ),
            WM_MBUTTONDOWN => mouse_input::<T>(
                handle,
                MouseButton::Middle,
                KeyState::Pressed,
                wparam,
                lparam,
            ),
            WM_XBUTTONDOWN => mouse_input::<T>(
                handle,
                wparam_to_button(wparam),
                KeyState::Pressed,
                wparam,
                lparam,
            ),
            WM_LBUTTONUP => mouse_input::<T>(
                handle,
                MouseButton::Left,
                KeyState::Released,
                wparam,
                lparam,
            ),
            WM_RBUTTONUP => mouse_input::<T>(
                handle,
                MouseButton::Right,
                KeyState::Released,
                wparam,
                lparam,
            ),
            WM_MBUTTONUP => mouse_input::<T>(
                handle,
                MouseButton::Middle,
                KeyState::Released,
                wparam,
                lparam,
            ),
            WM_XBUTTONUP => mouse_input::<T>(
                handle,
                wparam_to_button(wparam),
                KeyState::Released,
                wparam,
                lparam,
            ),
            WM_KEYDOWN => key_input::<T>(handle, KeyState::Pressed, wparam, lparam),
            WM_KEYUP => key_input::<T>(handle, KeyState::Released, wparam, lparam),
            WM_CHAR => {
                call_handler(|eh: &mut T, _| {
                    if let Some(c) = std::char::from_u32(wparam.0 as u32) {
                        eh.char_input(handle, c);
                    }
                });
                LRESULT(0)
            }
            WM_IME_SETCONTEXT => {
                let lparam = {
                    let state = handle.state.read().unwrap();
                    let mut lparam = lparam.0 as u32;
                    if !state.visible_ime_composition_window {
                        lparam &= !ISC_SHOWUICOMPOSITIONWINDOW;
                    }
                    if !state.visible_ime_candidate_window {
                        lparam &= !ISC_SHOWUICANDIDATEWINDOW;
                        lparam &= !(ISC_SHOWUICANDIDATEWINDOW << 1);
                        lparam &= !(ISC_SHOWUICANDIDATEWINDOW << 2);
                        lparam &= !(ISC_SHOWUICANDIDATEWINDOW << 3);
                    }
                    LPARAM(lparam as _)
                };
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_IME_STARTCOMPOSITION => {
                let imc = ime::Imc::get(hwnd);
                let state = handle.state.read().unwrap();
                if state.visible_ime_composition_window {
                    imc.set_composition_window_position(state.ime_position);
                }
                if state.visible_ime_candidate_window {
                    imc.set_candidate_window_position(
                        state.ime_position,
                        state.visible_ime_composition_window,
                    );
                }
                call_handler(|eh: &mut T, _| {
                    eh.ime_start_composition(handle);
                });
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_IME_COMPOSITION => {
                call_handler(|eh: &mut T, _| {
                    let imc = ime::Imc::get(hwnd);
                    if (lparam.0 as u32) & GCS_COMPSTR != 0 {
                        if let Some(ime::CompositionString::CompStr(s)) =
                            imc.get_composition_string(GCS_COMPSTR)
                        {
                            if let Some(ime::CompositionString::CompAttr(attrs)) =
                                imc.get_composition_string(GCS_COMPATTR)
                            {
                                eh.ime_composition(
                                    handle,
                                    &ime::Composition::new(s, attrs),
                                    imc.get_candidate_list().as_ref(),
                                );
                            }
                        }
                    }
                    if (lparam.0 as u32) & GCS_RESULTSTR != 0 {
                        if let Some(ime::CompositionString::ResultStr(s)) =
                            imc.get_composition_string(GCS_RESULTSTR)
                        {
                            if let Some(ime::CompositionString::CompAttr(attrs)) =
                                imc.get_composition_string(GCS_COMPATTR)
                            {
                                eh.ime_composition(handle, &ime::Composition::new(s, attrs), None);
                            }
                        }
                    }
                });
                let show_composition_window = {
                    let state = handle.state.read().unwrap();
                    state.visible_ime_composition_window
                };
                if show_composition_window {
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                } else {
                    LRESULT(0)
                }
            }
            WM_IME_ENDCOMPOSITION => {
                call_handler(|eh: &mut T, _| {
                    let imc = ime::Imc::get(hwnd);
                    let ret = imc.get_composition_string(GCS_RESULTSTR);
                    let ret = if let Some(ime::CompositionString::ResultStr(s)) = &ret {
                        Some(s.as_str())
                    } else {
                        None
                    };
                    eh.ime_end_composition(handle, ret);
                });
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_ACTIVATE => {
                if ((wparam.0 as u32) & WA_ACTIVE) != 0 || ((wparam.0 as u32) & WA_CLICKACTIVE) != 0
                {
                    call_handler(|eh: &mut T, _| eh.activated(handle));
                } else {
                    call_handler(|eh: &mut T, _| eh.inactivated(handle));
                }
                LRESULT(0)
            }
            WM_SIZE => {
                let value = lparam.0 as u32;
                let size = PhysicalSize::new(loword(value as _) as u32, hiword(value as _) as u32);
                call_handler(|eh: &mut T, state| {
                    if state.resizing {
                        eh.resizing(handle, size);
                    } else {
                        eh.resized(handle, size);
                    }
                });
                LRESULT(0)
            }
            WM_WINDOWPOSCHANGED => {
                let pos = &*(lparam.0 as *const WINDOWPOS);
                if pos.flags.0 & SWP_NOMOVE.0 == 0 {
                    call_handler(|eh: &mut T, _| {
                        eh.moved(handle, ScreenPosition::new(pos.x, pos.y))
                    });
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_ENTERSIZEMOVE => {
                set_resizing(true);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_EXITSIZEMOVE => {
                set_resizing(false);
                let size = handle.inner_size();
                call_handler(|eh: &mut T, _| eh.resized(handle, size));
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_DPICHANGED => {
                let rc = *(lparam.0 as *const RECT);
                SetWindowPos(
                    hwnd,
                    HWND(0),
                    rc.left,
                    rc.top,
                    rc.right - rc.left,
                    rc.bottom - rc.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
                call_handler(|eh: &mut T, _| eh.dpi_changed(handle));
                LRESULT(0)
            }
            WM_GETDPISCALEDSIZE => {
                let prev_dpi = GetDpiForWindow(hwnd) as i32;
                let next_dpi = wparam.0 as i32;
                let mut rc = RECT::default();
                GetClientRect(hwnd, &mut rc);
                let size = PhysicalSize::new(
                    ((rc.right - rc.left) * next_dpi / prev_dpi) as u32,
                    ((rc.bottom - rc.top) * next_dpi / prev_dpi) as u32,
                );
                let rc = adjust_window_rect(
                    size,
                    GetWindowLongPtrW(hwnd, GWL_STYLE) as _,
                    GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as _,
                    next_dpi as u32,
                );
                let mut ret = (lparam.0 as *mut SIZE).as_mut().unwrap();
                ret.cx = rc.right - rc.left;
                ret.cy = rc.bottom - rc.top;
                LRESULT(1)
            }
            WM_DROPFILES => {
                let hdrop = HDROP(wparam.0 as _);
                let file_count = DragQueryFileW(hdrop, std::u32::MAX, PWSTR::NULL, 0);
                let mut buffer = Vec::new();
                let files = (0..file_count)
                    .map(|i| {
                        let len = DragQueryFileW(hdrop, i, PWSTR::NULL, 0) as usize + 1;
                        buffer.resize(len, 0);
                        DragQueryFileW(hdrop, i, PWSTR(buffer.as_mut_ptr()), len as u32);
                        buffer.pop();
                        PathBuf::from(String::from_utf16_lossy(&buffer))
                    })
                    .collect::<Vec<_>>();
                let files_ref = files.iter().map(|pb| pb.as_path()).collect::<Vec<_>>();
                let mut pt = POINT::default();
                DragQueryPoint(hdrop, &mut pt);
                call_handler(|eh: &mut T, _| {
                    eh.drop_files(
                        handle,
                        &files_ref,
                        PhysicalPosition::new(pt.x as f32, pt.y as f32),
                    );
                });
                DragFinish(hdrop);
                LRESULT(0)
            }
            #[cfg(feature = "raw_input")]
            WM_INPUT_DEVICE_CHANGE => {
                raw_input::wm_input_device_change::<T>(handle, hwnd, wparam, lparam)
            }
            WM_DESTROY => {
                {
                    let mut state = handle.state.write().unwrap();
                    state.closed = true;
                }
                call_handler(|eh: &mut T, _| {
                    eh.closed(handle);
                    {
                        let state = handle.state.read().unwrap();
                        for child in state.children.iter() {
                            child.close();
                        }
                    }
                });
                remove_window(hwnd);
                if window_table_is_empty() {
                    PostQuitMessage(0);
                }
                LRESULT(0)
            }
            WM_NCCREATE => {
                EnableNonClientDpiScaling(hwnd);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_USER => {
                match wparam.0 {
                    w if w == UserMessage::SetTitle as usize => {
                        let state = handle.state.read().unwrap();
                        SetWindowTextW(hwnd, state.title.as_str());
                    }
                    w if w == UserMessage::SetPosition as usize => {
                        let state = handle.state.read().unwrap();
                        SetWindowPos(
                            hwnd,
                            HWND(0),
                            state.set_position.0,
                            state.set_position.1,
                            0,
                            0,
                            SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE,
                        );
                    }
                    w if w == UserMessage::SetInnerSize as usize => {
                        let state = handle.state.read().unwrap();
                        let rc = adjust_window_rect(
                            state.set_inner_size,
                            GetWindowLongPtrW(hwnd, GWL_STYLE) as _,
                            GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as _,
                            GetDpiForWindow(hwnd),
                        );
                        SetWindowPos(
                            hwnd,
                            HWND(0),
                            0,
                            0,
                            rc.right - rc.left,
                            rc.bottom - rc.top,
                            SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE,
                        );
                    }
                    w if w == UserMessage::EnableIme as usize => {
                        window.ime_context.borrow().enable();
                    }
                    w if w == UserMessage::DisableIme as usize => {
                        window.ime_context.borrow().disable();
                    }
                    w if w == UserMessage::SetStyle as usize => {
                        let style = {
                            let state = handle.state.read().unwrap();
                            state.style
                        };
                        let rc = adjust_window_rect(
                            handle.inner_size().to_physical(handle.dpi()),
                            style,
                            0,
                            GetDpiForWindow(hwnd),
                        );
                        SetWindowLongPtrW(hwnd, GWL_STYLE, style as _);
                        SetWindowPos(
                            hwnd,
                            HWND(0),
                            0,
                            0,
                            rc.right - rc.left,
                            rc.bottom - rc.top,
                            SWP_NOMOVE | SWP_NOZORDER | SWP_FRAMECHANGED,
                        );
                        ShowWindow(hwnd, SW_SHOW);
                    }
                    w if w == UserMessage::AcceptDragFiles as usize => {
                        DragAcceptFiles(hwnd, BOOL(lparam.0 as _));
                    }
                    _ => unreachable!(),
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    });
    ret.unwrap_or_else(|e| {
        set_unwind(e);
        LRESULT(0)
    })
}
