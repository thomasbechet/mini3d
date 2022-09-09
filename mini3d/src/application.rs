use crate::asset::{AssetManager, Asset};
use crate::asset::font::Font;
use crate::event_recorder::EventRecorder;
use crate::graphics::Graphics;
use crate::input::input_manager::InputManager;

pub struct Application {
    pub graphics: Graphics,
    pub asset_manager: AssetManager,
    input_manager: InputManager,
    count: usize,
}

impl Default for Application {
    fn default() -> Self {
        let mut app = Self {
            graphics: Default::default(),
            asset_manager: Default::default(),
            input_manager: Default::default(),
            count: 0,
        };
        app.asset_manager.fonts.insert(0, Asset::<Font> { name: "default", id: 0, resource: Default::default() });
        app
    }
}

impl Application {

    pub fn progress(&mut self, event_recorder: &EventRecorder) {

        self.graphics.commands.clear();

        // Prepare input manager
        self.input_manager.prepare_dispatch();
        // Dispatch input events
        for event in &event_recorder.input_events {
            self.input_manager.dispatch_event(event);
        }

        // TODO: dispatch more events ...

        // Update input layout
        self.input_manager.update();
        self.input_manager.render(&mut self.graphics);


        self.graphics.print((8, 8).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), 0);
        self.graphics.print((8, 32).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), 0);
        self.graphics.print((8, 52).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), 0);
        self.graphics.print((8, 70).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), 0);
        self.graphics.print((8, 100).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), 0);
        self.graphics.print((8, 150).into(), format!("{} {{|}}~éèê!\"#$%&\'()*+,-./:;<=>?[]^_`", self.count).as_str(), 0);
        self.graphics.print((8, 170).into(), format!("{} if self.is_defined() [], '''", self.count).as_str(), 0);
        self.count += 1;

    }
}