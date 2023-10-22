pub(crate) enum ResourceAddedHook {
    Renderer(RendererResourceHook),
    Input(InputResourceHook),
}

pub(crate) enum ResourceRemovedHook {
    Renderer(RendererResourceHook),
    Input(InputResourceHook),
}

pub(crate) enum RendererResourceHook {
    Texture,
}
pub(crate) enum InputResourceHook {
    Action,
    Axis,
    Text,
}
