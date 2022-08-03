use crate::asset::{AssetManager, Asset};
use crate::asset::font::Font;
use crate::event::Event;
use crate::input::input_table::InputTable;
use crate::service::renderer::RendererService;

pub struct App {
    event_queue: Vec<Event>,
    input_table: InputTable,
    asset_manager: AssetManager,
    default_font: Asset<Font>,
    count: usize,
}

impl App {

    pub fn new() -> Self {
        App { 
            event_queue: Vec::new(),
            input_table: Default::default(),
            asset_manager: AssetManager::new(),
            default_font: Asset::<Font> { name: "default", id: 0, resource: Default::default() },
            count: 0,
        }
    }

    pub fn push_event(&mut self, event: Event) {
        self.event_queue.push(event);
    }

    fn dispatch_events(&mut self) {
        for event in &self.event_queue {
            match event {
                Event::CloseRequested => {

                },
                Event::Input(input_event) => { 
                    self.input_table.dispatch_event(input_event);
                },
                Event::AssetImport(asset_import_event) => {
                    self.asset_manager.dispatch_event(asset_import_event);
                },
            }
        }
        self.event_queue.clear();
    }

    pub fn progress(&mut self) {

        // Update input table
        self.input_table.update_inputs();

        // Dispatch all application events
        self.dispatch_events();
    }

    pub fn render(&mut self, renderer: &mut impl RendererService) {
        renderer.clear();
        renderer.print(8, 8, format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), &self.default_font);
        renderer.print(8, 32, format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), &self.default_font);
        renderer.print(8, 52, format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), &self.default_font);
        renderer.print(8, 70, format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), &self.default_font);
        renderer.print(8, 100, format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), &self.default_font);
        renderer.print(8, 150, format!("{} This is a very important message from Thomas !!!", self.count).as_str(), &self.default_font);
        renderer.print(8, 170, format!("{} if self.is_defined() [], '''", self.count).as_str(), &self.default_font);
        renderer.draw_line(0, self.count as u16 % 200, self.count as u16 % 200, 100);
        renderer.fill_rect(100, 150, 150, 200);
        renderer.draw_vline(220, 50, 100);
        renderer.draw_hline(150, 200, 250);
        renderer.draw_rect(250, 200, 400, 300);
        self.count += 1;
    }
}