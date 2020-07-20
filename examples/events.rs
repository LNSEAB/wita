use wita::{Context, EventHandler, KeyCode, KeyState, MouseButton, MouseState, RunType, Window, WindowBuilder};

struct Application;

impl EventHandler for Application {
    fn activated(&mut self, _: &Window) {
        println!("activated");
    }

    fn inactivated(&mut self, _: &Window) {
        println!("inactivated");
    }

    fn closed(&mut self, _: &Window) {
        println!("closed");
    }

    fn dpi_changed(&mut self, window: &Window) {
        println!("dpi changed: {}", window.scale_factor());
    }

    fn mouse_input(&mut self, _: &Window, button: MouseButton, button_state: KeyState, mouse_state: MouseState) {
        println!("mouse_input: {:?}, {:?}, {:?}", button, button_state, mouse_state);
    }

    fn cursor_moved(&mut self, _: &Window, state: MouseState) {
        println!("cursor moved: {:?}", state);
    }

    fn cursor_entered(&mut self, _: &Window, state: MouseState) {
        println!("cursor entered: {:?}", state);
    }

    fn cursor_leaved(&mut self, _: &Window, state: MouseState) {
        println!("cursor leaved: {:?}", state);
    }

    fn key_input(&mut self, _: &Window, code: KeyCode, state: KeyState) {
        println!("key input: {:?}, {:?}", code, state);
    }
}

fn main() {
    let context = Context::new();
    let _window = WindowBuilder::new().title("wita events").build(&context);
    context.run(RunType::Wait, Application);
}
