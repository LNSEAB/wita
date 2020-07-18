use wita::{
    Context, EventHandler, KeyState, MouseButton, MouseState, RunType, Window, WindowBuilder,
};

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

    fn mouse_input(
        &mut self,
        _: &Window,
        button: MouseButton,
        button_state: KeyState,
        mouse_state: MouseState,
    ) {
        println!(
            "mouse_input: {:?}, {:?}, {:?}",
            button, button_state, mouse_state
        );
    }
}

fn main() {
    let context = Context::new();
    let _window = WindowBuilder::new().title("wita events").build(&context);
    context.run(RunType::Wait, Application);
}
