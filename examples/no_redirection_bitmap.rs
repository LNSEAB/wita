struct Application {
    frame: bool,
}

impl Application {
    fn new() -> anyhow::Result<Self> {
        wita::WindowBuilder::new()
            .title("wita no redirection bitmap")
            .no_redirection_bitmap(true)
            .build()?;
        Ok(Self { frame: true })
    }
}

impl wita::EventHandler for Application {
    fn key_input(
        &mut self,
        window: &wita::Window,
        code: wita::KeyCode,
        state: wita::KeyState,
        _: bool,
    ) {
        if state == wita::KeyState::Pressed {
            match code.vkey {
                wita::VirtualKey::Char('Q') => window.close(),
                wita::VirtualKey::Char('T') => {
                    if !self.frame {
                        window.set_style(wita::WindowStyle::dialog());
                    } else {
                        window.set_style(wita::WindowStyle::borderless());
                    }
                    println!("{:?}", window.position());
                    self.frame = !self.frame;
                }
                _ => (),
            }
        }
    }
}

fn main() {
    wita::run(wita::RunType::Wait, Application::new).unwrap();
}
