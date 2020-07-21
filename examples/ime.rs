struct Application {
    enabled_ime: bool,
}

impl Application {
    fn new() -> Self {
        Self {
            enabled_ime: true,
        }
    }
}

impl wita::EventHandler for Application {
    fn key_input(&mut self, window: &wita::Window, code: wita::KeyCode, state: wita::KeyState) {
        if code.vkey == wita::VirtualKey::Char('T') && state == wita::KeyState::Released {
            if self.enabled_ime {
                window.disable_ime();
                self.enabled_ime = false;
                println!("disabled ime");
            } else {
                window.enable_ime();
                self.enabled_ime = true;
                println!("enabled ime");
            }
        }
    }

    fn ime_start_composition(&mut self, _: &wita::Window) {
        println!("ime start composition");
    }

    fn ime_composition(&mut self, _: &wita::Window, comp: &wita::ime::Composition, candidate: Option<&wita::ime::CandidateList>) {
        println!("ime composition: {:?} {:?}", comp, candidate);
    }

    fn ime_end_composition(&mut self, _: &wita::Window, s: Option<&str>) {
        println!("ime end composition: {:?}", s);
    }
}

fn main() {
    let context = wita::Context::new();
    let window = wita::WindowBuilder::new()
        .title("wita ime")
        .visible_composition_window(true)
        .visible_candidate_window(false)
        .build(&context);
    window.set_ime_position(wita::LogicalPosition::new(100.0, 100.0));
    context.run(wita::RunType::Wait, Application::new());
}
