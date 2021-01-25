use wita::ToLogicalPosition;

struct Application;

impl Application {
    fn new() -> Self {
        wita::WindowBuilder::new().title("wita events").build();
        Self
    }
}

impl wita::EventHandler for Application {
    fn draw(&mut self, _: &wita::Window) {
        println!("draw");
    }

    fn activated(&mut self, _: &wita::Window) {
        println!("activated");
    }

    fn inactivated(&mut self, _: &wita::Window) {
        println!("inactivated");
    }

    fn closed(&mut self, _: &wita::Window) {
        println!("closed");
    }

    fn resizing(&mut self, _: &wita::Window, size: wita::PhysicalSize<u32>) {
        println!("resizing: {:?}", size);
    }

    fn resized(&mut self, _: &wita::Window, size: wita::PhysicalSize<u32>) {
        println!("resized: {:?}", size);
    }

    fn moved(&mut self, _: &wita::Window, pt: wita::ScreenPosition) {
        println!("moved: {:?}", pt);
    }

    fn dpi_changed(&mut self, window: &wita::Window) {
        println!("dpi changed: {}", window.scale_factor());
    }

    fn mouse_input(
        &mut self,
        _: &wita::Window,
        button: wita::MouseButton,
        button_state: wita::KeyState,
        mouse_state: wita::MouseState,
    ) {
        println!(
            "mouse_input: {:?}, {:?}, {:?}",
            button, button_state, mouse_state
        );
    }

    fn cursor_moved(&mut self, wnd: &wita::Window, state: wita::MouseState) {
        println!(
            "cursor moved: {:?} {:?}",
            state,
            state.position.to_logical(wnd.dpi() as i32)
        );
    }

    fn cursor_entered(&mut self, _: &wita::Window, state: wita::MouseState) {
        println!("cursor entered: {:?}", state);
    }

    fn cursor_leaved(&mut self, _: &wita::Window, state: wita::MouseState) {
        println!("cursor leaved: {:?}", state);
    }

    fn key_input(
        &mut self,
        _: &wita::Window,
        code: wita::KeyCode,
        state: wita::KeyState,
        prev_pressed: bool,
    ) {
        println!("key input: {:?}, {:?}, {}", code, state, prev_pressed);
    }

    fn char_input(&mut self, _: &wita::Window, c: char) {
        if c.is_control() || c.is_whitespace() {
            println!("char input: 0x{:02x}", c as u16);
        } else {
            println!("char input: {}", c);
        }
    }

    fn ime_start_composition(&mut self, _: &wita::Window) {
        println!("ime start composition");
    }

    fn ime_composition(
        &mut self,
        _: &wita::Window,
        comp: &wita::ime::Composition,
        candidate: Option<&wita::ime::CandidateList>,
    ) {
        println!("ime composition: {:?} {:?}", comp, candidate);
    }

    fn ime_end_composition(&mut self, _: &wita::Window, s: Option<&str>) {
        println!("ime end composition: {:?}", s);
    }
}

fn main() {
    wita::initialize::<Application>();
    wita::run(wita::RunType::Wait, Application::new());
}
