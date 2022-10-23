pub struct LifecycleComponent {
    pub alive: bool,
    pub enabled: bool,
}

impl Default for LifecycleComponent {
    fn default() -> Self {
        Self { alive: true, enabled: true, }
    }
}