use crate::platform::renderer::Renderer;
use crate::platform::window::Window;

#[derive(Default)]
pub struct App {
    window: Option<Box<dyn Window>>,
    renderer: Option<Box<dyn Renderer>>,
    runner: Option<impl Fn(App)>,
}

impl App {

    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn with_window(&mut self, window: Box<dyn Window>) -> &mut Self {
        self.window = Some(window);
        self
    }
    pub fn with_renderer(&mut self, renderer: Box<dyn Renderer>) -> &mut Self {
        self.renderer = Some(renderer);
        self
    }
    pub fn with_runner(&mut self, runner: impl Fn(App)) -> &mut Self {
        self.runner = Some(Box::new(runner));
        self
    }

    pub fn update(&mut self) {

    }

    pub fn run(mut self) {
        // if let Some(runner) = self.runner.as_mut() {
            // runner.invoke(self)
        // }

        self.runner.unwrap().invoke(self)
    }
}