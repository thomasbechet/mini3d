use crate::asset::{AssetManager, Asset};
use crate::asset::font::Font;
use crate::event::Event;
use crate::input::input_manager::InputManager;
use crate::service::renderer::RendererService;

pub struct App {
    event_queue: Vec<Event>,
    input_manager: InputManager,
    asset_manager: AssetManager,
    default_font: Asset<Font>,
    count: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            event_queue: Vec::new(),
            input_manager: Default::default(),
            asset_manager: Default::default(),
            default_font: Asset::<Font> { name: "default", id: 0, resource: Default::default() },
            count: 0,
        }
    }
}

impl App {
    pub fn push_event(&mut self, event: Event) {
        self.event_queue.push(event);
    }

    pub fn pull_events(&self) {

    }

    fn dispatch_events(&mut self) {
        for event in &self.event_queue {
            match event {
                Event::CloseRequested => {

                },
                Event::Input(input_event) => { 
                    self.input_manager.dispatch_event(input_event);
                },
                Event::AssetImport(asset_import_event) => {
                    self.asset_manager.dispatch_event(asset_import_event);
                },
            }
        }
        self.event_queue.clear();
    }

    pub fn progress(&mut self) {

        // Prepare input manager
        self.input_manager.prepare_dispatch();

        // Dispatch all application events
        self.dispatch_events();

        // Update input layout
        self.input_manager.update();
    }

    pub fn render(&mut self, renderer: &mut impl RendererService) {
        renderer.clear();
        renderer.print((8, 8).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), &self.default_font);
        renderer.print((8, 32).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), &self.default_font);
        renderer.print((8, 52).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), &self.default_font);
        renderer.print((8, 70).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), &self.default_font);
        renderer.print((8, 100).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), &self.default_font);
        renderer.print((8, 150).into(), format!("{} Ceci est un message trèéês important olala !!!", self.count).as_str(), &self.default_font);
        renderer.print((8, 170).into(), format!("{} if self.is_defined() [], '''", self.count).as_str(), &self.default_font);
        self.count += 1;

        self.input_manager.render(renderer);
    }
}