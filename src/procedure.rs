use crate::{api::*, context::*, device::*, event::EventHandler, geometry::*, ime, window::Window};
use std::panic::catch_unwind;
use std::path::PathBuf;
use winapi::shared::{minwindef::*, windef::*, windowsx::*};
use winapi::um::{shellapi::*, winuser::*};

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

fn lparam_to_point(lparam: LPARAM) -> PhysicalPosition<i32> {
    PhysicalPosition::new(GET_X_LPARAM(lparam) as i32, GET_Y_LPARAM(lparam) as i32)
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
    0
}

fn key_input<T: EventHandler + 'static>(
    window: &Window,
    state: KeyState,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let scan_code = ScanCode(((lparam >> 16) & 0x7f) as u32);
    call_handler(|eh: &mut T, _| {
        eh.key_input(
            window,
            KeyCode::new(as_virtual_key(wparam as i32), scan_code),
            state,
            (lparam >> 30) & 0x01 != 0,
        );
    });
    0
}

pub(crate) unsafe extern "system" fn window_proc<T: EventHandler + 'static>(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ret = catch_unwind(|| {
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
                0
            }
            WM_MOUSEMOVE => {
                call_handler(|eh: &mut T, state| {
                    let position = lparam_to_point(lparam);
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
                0
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
                0
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
                    std::char::from_u32(wparam as u32).map(|c| eh.char_input(handle, c));
                });
                0
            }
            WM_IME_SETCONTEXT => {
                let lparam = {
                    let state = handle.state.read().unwrap();
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
                let imc = Imc::get(hwnd);
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
                    let imc = Imc::get(hwnd);
                    if lparam & GCS_COMPSTR as LPARAM != 0 {
                        if let Some(CompositionString::CompStr(s)) =
                            imc.get_composition_string(GCS_COMPSTR)
                        {
                            if let Some(CompositionString::CompAttr(attrs)) =
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
                    if lparam & GCS_RESULTSTR as LPARAM != 0 {
                        if let Some(CompositionString::ResultStr(s)) =
                            imc.get_composition_string(GCS_RESULTSTR)
                        {
                            if let Some(CompositionString::CompAttr(attrs)) =
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
                    0
                }
            }
            WM_IME_ENDCOMPOSITION => {
                call_handler(|eh: &mut T, _| {
                    let imc = Imc::get(hwnd);
                    let ret = imc.get_composition_string(GCS_RESULTSTR);
                    let ret = if let Some(CompositionString::ResultStr(s)) = &ret {
                        Some(s.as_str())
                    } else {
                        None
                    };
                    eh.ime_end_composition(handle, ret);
                });
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_ACTIVATE => {
                if (wparam & WA_ACTIVE as WPARAM) != 0 || (wparam & WA_CLICKACTIVE as WPARAM) != 0 {
                    call_handler(|eh: &mut T, _| eh.activated(handle));
                } else {
                    call_handler(|eh: &mut T, _| eh.inactivated(handle));
                }
                0
            }
            WM_SIZE => {
                let value = lparam as DWORD;
                let size = PhysicalSize::new(LOWORD(value) as u32, HIWORD(value) as u32);
                call_handler(|eh: &mut T, state| {
                    if state.resizing {
                        eh.resizing(handle, size);
                    } else {
                        eh.resized(handle, size);
                    }
                });
                0
            }
            WM_WINDOWPOSCHANGED => {
                let pos = &*(lparam as *const WINDOWPOS);
                if pos.flags & SWP_NOMOVE == 0 {
                    call_handler(|eh: &mut T, _| {
                        eh.moved(handle, ScreenPosition::new(pos.x, pos.y))
                    });
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_ENTERSIZEMOVE => {
                context_mut(|ctx| ctx.state_mut().resizing = true);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_EXITSIZEMOVE => {
                context_mut(|ctx| ctx.state_mut().resizing = false);
                let size = handle.inner_size();
                call_handler(|eh: &mut T, _| eh.resized(handle, size));
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
                call_handler(|eh: &mut T, _| eh.dpi_changed(handle));
                0
            }
            WM_GETDPISCALEDSIZE => {
                let prev_dpi = GetDpiForWindow(hwnd) as i32;
                let next_dpi = wparam as i32;
                let mut rc = RECT::default();
                GetClientRect(hwnd, &mut rc);
                let size = PhysicalSize::new(
                    ((rc.right - rc.left) * next_dpi / prev_dpi) as u32,
                    ((rc.bottom - rc.top) * next_dpi / prev_dpi) as u32,
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
            WM_DROPFILES => {
                let hdrop = wparam as HDROP;
                let file_count = DragQueryFileW(hdrop, UINT::MAX, std::ptr::null_mut(), 0);
                let mut buffer = Vec::new();
                let files = (0..file_count)
                    .map(|i| {
                        let len = DragQueryFileW(hdrop, i, std::ptr::null_mut(), 0) as usize + 1;
                        buffer.resize(len, 0);
                        DragQueryFileW(hdrop, i, buffer.as_mut_ptr(), len as u32);
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
                0
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
                0
            }
            WM_NCCREATE => {
                EnableNonClientDpiScaling(hwnd);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_USER => {
                match wparam {
                    w if w == UserMessage::SetTitle as usize => {
                        let state = handle.state.read().unwrap();
                        let s = state
                            .title
                            .encode_utf16()
                            .chain(Some(0))
                            .collect::<Vec<_>>();
                        SetWindowTextW(hwnd, s.as_ptr());
                    }
                    w if w == UserMessage::SetPosition as usize => {
                        let state = handle.state.read().unwrap();
                        SetWindowPos(
                            hwnd,
                            std::ptr::null_mut(),
                            state.set_position.x,
                            state.set_position.y,
                            0,
                            0,
                            SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE,
                        );
                    }
                    w if w == UserMessage::SetInnerSize as usize => {
                        let state = handle.state.read().unwrap();
                        let rc = adjust_window_rect(
                            state.set_inner_size,
                            GetWindowLongPtrW(hwnd, GWL_STYLE) as u32,
                            GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32,
                            GetDpiForWindow(hwnd),
                        );
                        SetWindowPos(
                            hwnd,
                            std::ptr::null_mut(),
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
                        SetWindowLongPtrW(hwnd, GWL_STYLE, style as isize);
                        SetWindowPos(
                            hwnd,
                            std::ptr::null_mut(),
                            0,
                            0,
                            rc.right - rc.left,
                            rc.bottom - rc.top,
                            SWP_NOMOVE | SWP_NOZORDER | SWP_FRAMECHANGED,
                        );
                        ShowWindow(hwnd, SW_SHOW);
                    }
                    w if w == UserMessage::AcceptDragFiles as usize => {
                        DragAcceptFiles(hwnd, lparam as i32);
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
