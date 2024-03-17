use mini3d_derive::{component, fixed};
use mini3d::{
    api::API,
    component::{hierarchy::Hierarchy, texture::Texture, transform::Transform},
    db::query::Query,
    info,
    math::{fixed::I32F16, vec::V3I32F16},
    renderer::{provider::RendererProvider, texture::TextureHandle},
    Runtime, RuntimeConfig,
};
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

#[component]
struct MyComponent {
    handle: TextureHandle,
}

fn my_system(api: &mut API) {
    info!(api, "my_system");
    let transform = Transform::meta(api);
    let hierarchy = Hierarchy::meta(api);
    let my_tag = api.find_component("my_tag").unwrap();
    let query = Query::default().all(&[transform.id()]);
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

fn on_start(api: &mut API) {
    let transform = Transform::create_component(api);
    let hierarchy = Hierarchy::create_component(api);
    let my_tag = api.create_tag_component("my_tag").unwrap();
    let my_component = MyComponent::create_component(api);
    let texture = Texture::meta(api);
    let e = api.create();
    transform.add_from_translation(api, e, V3I32F16::ONE);
    api.add_default(e, hierarchy.id());
    api.debug_sched();
    api.add_default(e, my_tag);
    let e = api.create();
    api.add_default(e, my_tag);
    api.add_default(e, my_component.id());
    texture.add_default(api, e);
}

fn main() {
    let mut runtime = Runtime::new(RuntimeConfig::default().bootstrap(|api| {
        let texture = Texture::create_component(api);
        texture.create_callbacks(api);
        api.create_system("my_system", api.tick_stage(), Default::default(), my_system)
            .unwrap();
        api.create_system(
            "start_system",
            api.start_stage(),
            Default::default(),
            on_start,
        )
        .unwrap();
    }));
    runtime.set_logger(StdoutLogger);
    runtime.set_renderer(LoggerRenderer);
    for _ in 0..1 {
        runtime.tick().expect("Simulation error");
    }
    println!("target_tps: {}", runtime.target_tps());
    println!("DONE");
}
