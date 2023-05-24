use mini3d_derive::Component;

#[derive(Component)]
#[component(name = "life_cycle")]
pub struct Lifecycle {
    pub alive: bool,
}

impl Lifecycle {
    
    pub fn alive() -> Self {
        Self { alive: true }
    }

    pub fn dead() -> Self {
        Self { alive: false }
    }
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::alive()
    }
}