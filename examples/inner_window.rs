use winapi::um::d2d1::*;
use winapi::um::dcommon::*;
use winapi::shared::dxgiformat::*;
use winapi::Interface;
use com_ptr::*;

struct Application {
    root_wnd: wita::Window,
    d2d1_wnd: wita::Window,
    render_target: ComPtr<ID2D1HwndRenderTarget>,
}

impl Application {
    fn new() -> anyhow::Result<Self> {
        let root_wnd = wita::WindowBuilder::new()
            .title("wita inner window")
            .build()?;
        let d2d1_wnd = wita::InnerWindowBuilder::new()
            .parent(&root_wnd)
            .position(wita::LogicalPosition::new(10, 10))
            .size(wita::LogicalSize::new(320, 240))
            .build()?;
        let d2d1_factory = ComPtr::new(|| unsafe {
            let mut p = std::ptr::null_mut();
            let ret = D2D1CreateFactory(D2D1_FACTORY_TYPE_MULTI_THREADED, &ID2D1Factory::uuidof(), std::ptr::null(), &mut p);
            hresult(p as *mut ID2D1Factory, ret)
        })?;
        let dpi = d2d1_wnd.dpi() as f32;
        let render_target_size = d2d1_wnd.inner_size();
        let render_target = ComPtr::new(|| unsafe {
            let mut p = std::ptr::null_mut();
            let ret = d2d1_factory.CreateHwndRenderTarget(
                &D2D1_RENDER_TARGET_PROPERTIES {
                    _type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_R8G8B8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_UNKNOWN,
                    },
                    dpiX: dpi,
                    dpiY: dpi,
                    ..Default::default()
                },
                &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                    hwnd: d2d1_wnd.raw_handle() as _,
                    pixelSize: winapi::um::d2d1::D2D1_SIZE_U {
                        width: render_target_size.width,
                        height: render_target_size.height,
                    },
                    ..Default::default()
                },
                &mut p
            );
            hresult(p, ret)
        })?;
        Ok(Self {
            render_target,
            root_wnd,
            d2d1_wnd,
        })
    }
}

impl wita::EventHandler for Application {
    fn mouse_input(&mut self, wnd: &wita::Window, btn: wita::MouseButton, state: wita::KeyState, _: wita::MouseState) {
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
            self.render_target.Clear(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.3,
                a: 0.0,
            });
            self.render_target.EndDraw(std::ptr::null_mut(), std::ptr::null_mut());
        }
    }
}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
