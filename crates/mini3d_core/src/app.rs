use crate::asset::AssetManager;
use crate::event::Event;
use crate::input::input_table::{InputTable, AXIS_VIEW_X};
use crate::service::renderer::RendererService;

pub struct App {
    event_queue: Vec<Event>,
    input_table: InputTable,
    asset_manager: AssetManager,
}

impl App {

    pub fn new() -> Self {
        App { 
            event_queue: Vec::new(),
            input_table: Default::default(),
            asset_manager: AssetManager::new(),
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

    pub fn render(&mut self, renderer: &impl RendererService) {
        
    }
}