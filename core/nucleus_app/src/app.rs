use nucleus_renderer::Renderer;
use nucleus_window::Window;
use nucleus_runner::Runner;

pub struct PlatformProvider {
    window: Window,
    renderer: Renderer,
    runner: Option<Runner>,
}

#[derive(Default)]
pub struct App {
    platform: PlatformProvider
}

impl App {

    pub fn new(provider: PlatformProvider) -> Self {
        App {
            platform: provider
        }
    }

    pub fn update(&mut self) {

    }

    pub fn run(mut self) {
        if let Some(runner) = self.platform.runner {
            runner.invoke(self)
        }
    }
}