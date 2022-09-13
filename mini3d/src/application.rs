use crate::asset::AssetManager;
use crate::event::EventManager;
use crate::graphics::Graphics;
use crate::input::input_manager::InputManager;

pub struct Application {
    pub graphics: Graphics,
    pub events: EventManager,
    pub assets: AssetManager,
    inputs: InputManager,
    count: usize,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            graphics: Default::default(),
            events: Default::default(),
            assets: Default::default(),
            inputs: Default::default(),
            count: 0,
        }
    }
}

impl Application {

    pub fn progress(&mut self) {

        self.graphics.prepare();

        // Dispatch asset events
        for event in self.events.assets.drain(..) {
            self.assets.dispatch_event(event);
        }

        // Prepare input manager
        self.inputs.prepare_dispatch();
        // Dispatch input events
        for event in self.events.inputs.drain(..) {
            self.inputs.dispatch_event(&event);
        }

        // TODO: dispatch more events ...

        // Update input layout
        self.inputs.update();
        self.inputs.render(&mut self.graphics);


        self.graphics.print((8, 8).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), Default::default());
        self.graphics.print((8, 32).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), Default::default());
        self.graphics.print((8, 52).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), Default::default());
        self.graphics.print((8, 70).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), Default::default());
        self.graphics.print((8, 100).into(), format!("{} zefiozefjzoefijzeofijzoeifjâzpkeazêpfzeojfzoeijf", self.count).as_str(), Default::default());
        self.graphics.print((8, 150).into(), format!("{} {{|}}~éèê!\"#$%&\'()*+,-./:;<=>?[]^_`", self.count).as_str(), Default::default());
        self.graphics.print((8, 170).into(), format!("{} if self.is_defined() [], '''", self.count).as_str(), Default::default());
        self.count += 1;

    }
}