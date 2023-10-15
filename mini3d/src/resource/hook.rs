pub(crate) enum ResourceAddedHook {
    Renderer(RendererResourceHook),
}

pub(crate) enum ResourceRemovedHook {
    Renderer(RendererResourceHook),
}

pub(crate) enum RendererResourceHook {
    Texture,
}
