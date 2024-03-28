use mini3d::{
    api::API,
    component::{event::UserEvent, hierarchy::Hierarchy, input::InputAction, texture::Texture, transform::Transform},
    db::query::Query,
    event::Event,
    info,
    math::{fixed::I32F16, vec::V3I32F16},
    renderer::{provider::RendererProvider, texture::TextureId},
    Runtime, RuntimeConfig,
};
use mini3d_derive::{component, fixed};
use mini3d_stdlog::stdout::StdoutLogger;

fn on_transform_added(api: &mut API) {
    info!(api, "TRANSFORM ADDED")
}
fn on_transform_removed(api: &mut API) {
    info!(api, "TRANSFORM REMOVED")
}

struct LoggerRenderer;

impl RendererProvider for LoggerRenderer {
    fn create_texture(
        &mut self,
        data: &mini3d::renderer::texture::TextureData,
    ) -> Result<
        mini3d::renderer::provider::RendererProviderHandle,
        mini3d::renderer::provider::RendererProviderError,
    > {
        println!("create texture");
        Ok(Default::default())
    }
}

fn my_system2(api: &mut API, transform: &Transform) {}

fn my_system(api: &mut API, transform: &Transform, hierarchy: &Hierarchy) {
    info!(api, "my_system");
    let my_tag = api.find_component("my_tag").unwrap();
    let query = Query::default().all(&[transform]);
    info!(api, "hello world");
    let e = api.entities().next().unwrap();
    let mut pos = api.read(e, transform.translation).unwrap();
    info!(api, "{:?}", pos);
    pos.x += fixed!(0.5i32f16);
    api.write(e, transform.translation, pos);
    for e in api.entities() {
        api.dump(e);
    }
}

#[component]
struct MyComponent {}

fn on_start(
    api: &mut API,
    texture: &Texture,
    transform: &Transform,
    hierarchy: &Hierarchy,
    my_component: &MyComponent,
    user_event: &UserEvent,
    input_action: &InputAction,
) {
    let my_tag = api.find_component("my_tag").unwrap();
    let e = api.spawn();
    transform.add_from_translation(api, e, V3I32F16::ONE);
    api.add_default(e, hierarchy);
    api.debug_sched();
    api.add_default(e, my_tag);
    input_action.add(api, e, "my_input");
    let e = api.spawn();
    user_event.add(api, e, "my_event");
    api.add_default(e, my_tag);
    api.add_default(e, my_component);
    api.add_default(e, texture);
    api.add_default(e, input_action);
}

fn main() {
    let mut runtime = Runtime::new(RuntimeConfig::default().bootstrap(|api| {
        api.register_component_tag("my_tag");
        MyComponent::register(api);
        api.register_system("my_system2", Event::Tick, Default::default(), my_system2);
        api.register_system("my_system", Event::Tick, Default::default(), my_system);
        api.register_system("start_system", Event::Start, Default::default(), on_start);
    }));
    runtime.set_logger(StdoutLogger);
    runtime.set_renderer(LoggerRenderer);
    for _ in 0..1 {
        runtime.tick().expect("Simulation error");
    }
    println!("target_tps: {}", runtime.target_tps());
    println!("DONE");
}
