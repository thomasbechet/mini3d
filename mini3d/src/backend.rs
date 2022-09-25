use self::renderer::{RendererBackend, DefaultRendererBackend};

pub mod renderer;

pub struct BackendDescriptor<'a> {
    pub(crate) renderer: Option<&'a mut dyn RendererBackend>,
}

impl<'a> BackendDescriptor<'a> {

    pub fn new() -> Self {
        Self { renderer: None }
    }

    pub fn with_renderer<R: RendererBackend>(
        mut self, 
        renderer: &'a mut R
    ) -> Self {
        self.renderer = Some(renderer);
        self
    }
}

#[derive(Default)]
pub(crate) struct DefaultBackend {
    renderer: DefaultRendererBackend,
}

pub(crate) struct Backend<'a> {
    pub(crate) renderer: &'a mut dyn RendererBackend,
}

impl<'a> Backend<'a> {
    
    pub(crate) fn build( 
        descriptor: BackendDescriptor<'a>,
        default: &'a mut DefaultBackend,
    ) -> Self {
        let renderer = descriptor.renderer.unwrap_or(&mut default.renderer);
        Self {
            renderer,
        }
    }
}