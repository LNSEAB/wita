#[allow(warnings)]
mod bindings {
    ::windows::include_bindings!();
}

use bindings::windows::win32::{
    windows_and_messaging::*,
    direct2d::*,
    dxgi::*,
};
use windows::Abi;
use windows::Interface;

struct Application {
    root_wnd: wita::Window,
    d2d1_wnd: wita::Window,
    render_target: ID2D1HwndRenderTarget,
}

impl Application {
    fn new() -> anyhow::Result<Self> {
        let root_wnd = wita::WindowBuilder::new()
            .title("wita inner window")
            .ime(true)
            .build()?;
        let d2d1_wnd = wita::InnerWindowBuilder::new()
            .parent(&root_wnd)
            .position(wita::LogicalPosition::new(10, 10))
            .size(wita::LogicalSize::new(320, 240))
            .build()?;
        let d2d1_factory = unsafe {
            let mut p: Option<ID2D1Factory> = None;
            D2D1CreateFactory(
                D2D1_FACTORY_TYPE::D2D1_FACTORY_TYPE_MULTI_THREADED,
                &ID2D1Factory::IID,
                std::ptr::null(),
                p.set_abi(),
            ).and_some(p)?
        };
        let dpi = d2d1_wnd.dpi() as f32;
        let render_target_size = d2d1_wnd.inner_size();
        let render_target = unsafe {
            let mut p = None;
            d2d1_factory.CreateHwndRenderTarget(
                &D2D1_RENDER_TARGET_PROPERTIES {
                    r#type: D2D1_RENDER_TARGET_TYPE::D2D1_RENDER_TARGET_TYPE_DEFAULT,
                    pixel_format: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT::DXGI_FORMAT_R8G8B8A8_UNORM,
                        alpha_mode: D2D1_ALPHA_MODE::D2D1_ALPHA_MODE_UNKNOWN,
                    },
                    dpix: dpi,
                    dpiy: dpi,
                    ..Default::default()
                },
                &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                    hwnd: HWND(d2d1_wnd.raw_handle() as _),
                    pixel_size: D2D_SIZE_U {
                        width: render_target_size.width,
                        height: render_target_size.height,
                    },
                    ..Default::default()
                },
                &mut p,
            ).and_some(p)?
        };
        Ok(Self {
            render_target,
            root_wnd,
            d2d1_wnd,
        })
    }
}

impl wita::EventHandler for Application {
    fn mouse_input(
        &mut self,
        wnd: &wita::Window,
        btn: wita::MouseButton,
        state: wita::KeyState,
        _: wita::MouseState,
    ) {
        if btn == wita::MouseButton::Left && state == wita::KeyState::Pressed {
            if wnd == &self.root_wnd {
                println!("root_wnd");
            } else if wnd == &self.d2d1_wnd {
                println!("d2d1_wnd");
            }
        }
    }

    fn draw(&mut self, _: &wita::Window) {
        unsafe {
            self.render_target.BeginDraw();
            self.render_target.Clear(&DXGI_RGBA {
                r: 0.0,
                g: 0.0,
                b: 0.3,
                a: 0.0,
            });
            self.render_target
                .EndDraw(std::ptr::null_mut(), std::ptr::null_mut()).unwrap();
        }
    }
}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
