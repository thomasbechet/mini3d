use mini3d_core::{
    ecs::{
        api::{
            context::Context,
            ecs::ECS,
            input::Input,
            registry::{ComponentRegistry, ResourceRegistry, SystemRegistry},
            resource::Resource,
            runtime::Runtime,
        },
        scheduler::Invocation,
    },
    expect,
    feature::{
        common::{
            free_fly::FreeFly, hierarchy::Hierarchy, local_to_world::LocalToWorld,
            rotator::Rotator, script::Script, transform::Transform,
        },
        input::{
            action::InputAction,
            axis::{InputAxis, InputAxisRange},
        },
        renderer::{
            camera::Camera, font::Font, material::Material, mesh::Mesh, model::Model,
            staticmesh::StaticMesh, texture::Texture, viewport::Viewport,
        },
    },
    glam::{Quat, Vec3},
    info,
    platform::event::ImportAssetEvent,
    registry::{
        component::{ComponentStorage, StaticComponentType},
        resource::StaticResourceType,
        system::{ExclusiveSystem, SystemOrder, SystemStage},
    },
    renderer::{SCREEN_HEIGHT, SCREEN_RESOLUTION, SCREEN_WIDTH},
    script::{compiler::Compiler, module::Module},
    utils::prng::PCG32,
};

use crate::{
    component::os::OS,
    input::{CommonAction, CommonAxis},
    system::update::OSUpdate,
};

#[derive(Default)]
pub struct OSBootstrap;

impl OSBootstrap {
    pub const NAME: &'static str = "os_bootstrap";
}

impl ExclusiveSystem for OSBootstrap {
    fn run(&self, ecs: &mut ECS, ctx: &mut Context) {
        // Declare components
        expect!(
            ctx,
            ComponentRegistry::add_static::<OS>(ctx, OS::NAME, ComponentStorage::Single)
        );
        // Declare systems
        expect!(
            ctx,
            SystemRegistry::add_static_exclusive::<OSInitialize>(
                ctx,
                OSInitialize::NAME,
                OSInitialize::NAME,
                SystemOrder::default()
            )
        );
        expect!(
            ctx,
            SystemRegistry::add_static_exclusive::<OSUpdate>(
                ctx,
                OSUpdate::NAME,
                SystemStage::UPDATE,
                SystemOrder::default()
            )
        );
        // Initialize OS
        expect!(ctx, ecs.invoke(OSInitialize::NAME, Invocation::Immediate));
    }
}

#[derive(Default)]
pub struct OSInitialize;

impl OSInitialize {
    pub const NAME: &'static str = "os_initialize";
}

impl OSInitialize {
    fn setup_resources(&self, ctx: &mut Context) {
        // Register default font
        let font: StaticResourceType<Font> = ResourceRegistry::find(ctx, Font::NAME).unwrap();
        expect!(
            ctx,
            Resource::persist(ctx, font, "default", Font::default())
        );

        // Register inputs
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: CommonAction::CLICK.into(),
                    display_name: "Click".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: CommonAction::BACK.into(),
                    display_name: "Back".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: CommonAction::UP.into(),
                    display_name: "Up".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: CommonAction::LEFT.into(),
                    display_name: "Left".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: CommonAction::DOWN.into(),
                    display_name: "Down".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: CommonAction::RIGHT.into(),
                    display_name: "Right".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: CommonAction::CHANGE_CONTROL_MODE.into(),
                    display_name: "Change Control Mode".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: CommonAction::TOGGLE_PROFILER.into(),
                    display_name: "Toggle Profiler".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::CURSOR_X.into(),
                    display_name: "Cursor X".into(),
                    range: InputAxisRange::Clamped {
                        min: 0.0,
                        max: SCREEN_WIDTH as f32,
                    },
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::CURSOR_Y.into(),
                    display_name: "Cursor Y".into(),
                    range: InputAxisRange::Clamped {
                        min: 0.0,
                        max: SCREEN_HEIGHT as f32,
                    },
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::SCROLL_MOTION.into(),
                    display_name: "Scroll Motion".into(),
                    range: InputAxisRange::Infinite,
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::CURSOR_MOTION_X.into(),
                    display_name: "Cursor Motion X".into(),
                    range: InputAxisRange::Infinite,
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::CURSOR_MOTION_Y.into(),
                    display_name: "Cursor Motion Y".into(),
                    range: InputAxisRange::Infinite,
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::VIEW_X.into(),
                    display_name: "View X".into(),
                    range: InputAxisRange::Infinite,
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::VIEW_Y.into(),
                    display_name: "View Y".into(),
                    range: InputAxisRange::Infinite,
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::MOVE_FORWARD.into(),
                    display_name: "Move Forward".into(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::MOVE_BACKWARD.into(),
                    display_name: "Move Backward".into(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::MOVE_LEFT.into(),
                    display_name: "Move Left".into(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::MOVE_RIGHT.into(),
                    display_name: "Move Right".into(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::MOVE_UP.into(),
                    display_name: "Move Up".into(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                }
            )
        );
        expect!(
            ctx,
            Input::add_axis(
                ctx,
                InputAxis {
                    name: CommonAxis::MOVE_DOWN.into(),
                    display_name: "Move Down".into(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                }
            )
        );

        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: "roll_left".into(),
                    display_name: "Roll Left".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: "roll_right".into(),
                    display_name: "Roll Right".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: "switch_mode".into(),
                    display_name: "Switch Mode".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: "move_fast".into(),
                    display_name: "Move Fast".into(),
                    default_pressed: false,
                }
            )
        );
        expect!(
            ctx,
            Input::add_action(
                ctx,
                InputAction {
                    name: "move_slow".into(),
                    display_name: "Move Slow".into(),
                    default_pressed: false,
                }
            )
        );

        let texture: StaticResourceType<Texture> =
            expect!(ctx, ResourceRegistry::find(ctx, Texture::NAME));
        let mesh: StaticResourceType<Mesh> = expect!(ctx, ResourceRegistry::find(ctx, Mesh::NAME));
        let model: StaticResourceType<Model> =
            expect!(ctx, ResourceRegistry::find(ctx, Model::NAME));
        let material: StaticResourceType<Material> =
            expect!(ctx, ResourceRegistry::find(ctx, Material::NAME));
        let script: StaticResourceType<Script> =
            expect!(ctx, ResourceRegistry::find(ctx, Script::NAME));

        // Import resources
        while let Some(import) = Runtime::next_import(ctx) {
            match import {
                ImportAssetEvent::Material(entry) => {
                    expect!(
                        ctx,
                        Resource::persist(ctx, material, &entry.name, entry.data.clone())
                    );
                }
                ImportAssetEvent::Mesh(entry) => {
                    info!(ctx, "Importing mesh: {}", entry.name);
                    expect!(
                        ctx,
                        Resource::persist(ctx, mesh, &entry.name, entry.data.clone())
                    );
                }
                ImportAssetEvent::Model(entry) => {
                    expect!(
                        ctx,
                        Resource::persist(ctx, model, &entry.name, entry.data.clone())
                    );
                }
                ImportAssetEvent::Script(entry) => {
                    expect!(
                        ctx,
                        Resource::persist(ctx, script, &entry.name, entry.data.clone())
                    );
                }
                ImportAssetEvent::Texture(entry) => {
                    expect!(
                        ctx,
                        Resource::persist(ctx, texture, &entry.name, entry.data.clone())
                    );
                }
                _ => {}
            }
        }

        // Non default resources
        let alfred_texture = expect!(ctx, Resource::find(ctx, "alfred_tex"));
        let alfred_mesh = expect!(ctx, Resource::find(ctx, "alfred_mesh"));
        let car_texture = expect!(ctx, Resource::find(ctx, "car_tex"));
        let car_mesh = expect!(ctx, Resource::find(ctx, "car_mesh"));
        let alfred_material = expect!(
            ctx,
            Resource::persist(
                ctx,
                material,
                "alfred_mat",
                Material {
                    diffuse: alfred_texture,
                },
            )
        );
        let car_material = expect!(
            ctx,
            Resource::persist(
                ctx,
                material,
                "car_mat",
                Material {
                    diffuse: car_texture,
                },
            )
        );
        expect!(
            ctx,
            Resource::persist(
                ctx,
                model,
                "car_model",
                Model {
                    mesh: car_mesh,
                    materials: Vec::from([car_material]),
                },
            )
        );
        expect!(
            ctx,
            Resource::persist(
                ctx,
                model,
                "alfred_model",
                Model {
                    mesh: alfred_mesh,
                    materials: Vec::from([alfred_material, alfred_material, alfred_material]),
                },
            )
        );
    }

    fn setup_scene(&self, ecs: &mut ECS, ctx: &mut Context) {
        // Find components
        let transform: StaticComponentType<Transform> =
            expect!(ctx, ComponentRegistry::find(ctx, Transform::NAME));
        let rotator: StaticComponentType<Rotator> =
            expect!(ctx, ComponentRegistry::find(ctx, Rotator::NAME));
        let static_mesh: StaticComponentType<StaticMesh> =
            expect!(ctx, ComponentRegistry::find(ctx, StaticMesh::NAME));
        let local_to_world: StaticComponentType<LocalToWorld> =
            expect!(ctx, ComponentRegistry::find(ctx, LocalToWorld::NAME));
        let hierarchy: StaticComponentType<Hierarchy> =
            expect!(ctx, ComponentRegistry::find(ctx, Hierarchy::NAME));
        let camera: StaticComponentType<Camera> =
            expect!(ctx, ComponentRegistry::find(ctx, Camera::NAME));
        let viewport: StaticComponentType<Viewport> =
            expect!(ctx, ComponentRegistry::find(ctx, Viewport::NAME));
        // let ui: StaticComponent<UI> = expect!(ctx, ctx.registry.components.find(UI::NAME));
        // let ui_render_target: StaticComponent<UIRenderTarget> =
        //     expect!(ctx, ctx.registry.components.find(UIRenderTarget::NAME));
        let free_fly: StaticComponentType<FreeFly> =
            expect!(ctx, ComponentRegistry::find(ctx, FreeFly::NAME));
        let os: StaticComponentType<OS> = expect!(ctx, ComponentRegistry::find(ctx, OS::NAME));

        let alfred_model = expect!(ctx, Resource::find(ctx, "alfred_model"));
        // let alfred_texture = expect!(ctx, ctx.resource.find("alfred_tex"));
        let car_model = expect!(ctx, Resource::find(ctx, "car_model"));
        // let gui_texture = expect!(ctx, ctx.resource.find("GUI"));
        // let font = expect!(ctx, ctx.resource.find("default"));
        {
            let e = ecs
                .create()
                .with(
                    transform,
                    Transform {
                        translation: Vec3::new(0.0, -7.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(0.5, 0.5, 0.5),
                    },
                )
                .with_default(local_to_world)
                .with(rotator, Rotator { speed: 90.0 })
                .with(static_mesh, StaticMesh::new(alfred_model))
                .build();
        }
        {
            let e = ecs
                .create()
                .with(
                    transform,
                    Transform::from_translation(Vec3::new(0.0, -7.0, 9.0)),
                )
                .with_default(local_to_world)
                .with(static_mesh, StaticMesh::new(alfred_model))
                .build();
        }
        {
            let mut prng = PCG32::new(12345);
            for i in 0..100 {
                ecs.create()
                    .with(
                        transform,
                        Transform::from_translation(Vec3::new(
                            ((i / 10) * 5) as f32,
                            0.0,
                            -((i % 10) * 8) as f32,
                        )),
                    )
                    .with_default(local_to_world)
                    .with(static_mesh, StaticMesh::new(car_model))
                    .with(
                        rotator,
                        Rotator {
                            speed: -90.0 + prng.next_f32() * 90.0 * 2.0,
                        },
                    )
                    .build();

                ecs.create()
                    .with(
                        transform,
                        Transform::from_translation(Vec3::new(
                            ((i / 10) * 5) as f32,
                            10.0,
                            -((i % 10) * 8) as f32,
                        )),
                    )
                    .with_default(local_to_world)
                    .with(static_mesh, StaticMesh::new(alfred_model))
                    .with(
                        rotator,
                        Rotator {
                            speed: -90.0 + prng.next_f32() * 90.0 * 5.0,
                        },
                    )
                    .build();
            }
        }
        {
            ecs.create()
                .with(
                    transform,
                    Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
                )
                .with_default(local_to_world)
                .with(static_mesh, StaticMesh::new(car_model))
                .with(rotator, Rotator { speed: 30.0 })
                .build();
        }
        {
            let e = ecs
                .create()
                .with(
                    transform,
                    Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
                )
                .with_default(local_to_world)
                .with(
                    free_fly,
                    FreeFly {
                        active: true,
                        switch_mode: Input::find_action(ctx, "switch_mode").unwrap(),
                        roll_left: Input::find_action(ctx, "roll_left").unwrap(),
                        roll_right: Input::find_action(ctx, "roll_right").unwrap(),
                        view_x: Input::find_axis(ctx, CommonAxis::VIEW_X).unwrap(),
                        view_y: Input::find_axis(ctx, CommonAxis::VIEW_Y).unwrap(),
                        move_forward: Input::find_axis(ctx, CommonAxis::MOVE_FORWARD).unwrap(),
                        move_backward: Input::find_axis(ctx, CommonAxis::MOVE_BACKWARD).unwrap(),
                        move_up: Input::find_axis(ctx, CommonAxis::MOVE_UP).unwrap(),
                        move_down: Input::find_axis(ctx, CommonAxis::MOVE_DOWN).unwrap(),
                        move_left: Input::find_axis(ctx, CommonAxis::MOVE_LEFT).unwrap(),
                        move_right: Input::find_axis(ctx, CommonAxis::MOVE_RIGHT).unwrap(),
                        move_fast: Input::find_action(ctx, "move_fast").unwrap(),
                        move_slow: Input::find_action(ctx, "move_slow").unwrap(),
                        free_mode: false,
                        yaw: 0.0,
                        pitch: 0.0,
                    },
                )
                .with(static_mesh, StaticMesh::new(car_model))
                .with_default(hierarchy)
                .build();

            let cam = ecs
                .create()
                .with(
                    transform,
                    Transform::from_translation(Vec3::new(0.0, -1.0, 0.0)),
                )
                .with_default(local_to_world)
                .with(camera, Camera::default().with_fov(90.0))
                .with_default(hierarchy)
                .build();

            expect!(ctx, Hierarchy::attach(e, cam, &mut ecs.view_mut(hierarchy)));

            let viewport = ecs
                .create()
                .with(viewport, Viewport::new(SCREEN_RESOLUTION, Some(cam)))
                .build();

            //     let mut stylesheet = UIStyleSheet::empty();
            //     {
            //         let frame0 = UIBoxStyle::Image(
            //             UIImageStyle::sliced(
            //                 gui_texture,
            //                 IRect::new(2, 34, 44, 44),
            //                 UIMargin::new(6, 6, 6, 6),
            //             )
            //             .unwrap(),
            //         );
            //         let button_normal = UIBoxStyle::Image(
            //             UIImageStyle::sliced(
            //                 gui_texture,
            //                 IRect::new(1, 81, 14, 14),
            //                 UIMargin::new(3, 4, 3, 3),
            //             )
            //             .unwrap(),
            //         );
            //         let button_pressed = UIBoxStyle::Image(
            //             UIImageStyle::sliced(
            //                 gui_texture,
            //                 IRect::new(17, 81, 14, 14),
            //                 UIMargin::new(4, 3, 3, 3),
            //             )
            //             .unwrap(),
            //         );
            //         let button = UIButtonStyle::new(button_normal, button_pressed, button_normal);
            //         let checkbox_unchecked = UIBoxStyle::Image(
            //             UIImageStyle::sliced(
            //                 gui_texture,
            //                 IRect::new(81, 257, 14, 14),
            //                 UIMargin::new(3, 4, 3, 3),
            //             )
            //             .unwrap(),
            //         );
            //         let checkbox_checked = UIBoxStyle::Image(
            //             UIImageStyle::sliced(
            //                 gui_texture,
            //                 IRect::new(97, 257, 14, 14),
            //                 UIMargin::new(4, 3, 3, 3),
            //             )
            //             .unwrap(),
            //         );
            //         let checkbox = UICheckBoxStyle::new(checkbox_unchecked, checkbox_checked);
            //         stylesheet
            //             .add_button_style(UIButtonStyle::DEFAULT, button)
            //             .unwrap();
            //         stylesheet
            //             .add_checkbox_style(UICheckBoxStyle::DEFAULT, checkbox)
            //             .unwrap();
            //     }
            //     let mut gui = UI::new(SCREEN_RESOLUTION, stylesheet);
            //     // let box_style = UIBoxStyle::sliced("frame".into(), (0, 0, 96, 96).into(), UIMargin::new(5, 5, 5, 5), TextureWrapMode::Repeat);
            //     // let button_style = UIButtonStyle::new(box_style, box_style, box_style);
            //     let mut button = UIButton::new(IRect::new(10, 10, 60, 20));
            //     button.on_pressed(Some("HELLO".into()));
            //     let b0 = gui.root().add_button("b0", 5, button);
            //     // let b1 = ui.root().add_button("b1", 0, UIButton::new(IRect::new(10, 50, 50, 20)))?;
            //     gui.root().add_sprite(
            //         "alfred",
            //         1,
            //         UISprite::new(alfred_texture, (50, 50).into(), (0, 0, 64, 64).into()),
            //     );
            //     let t0 =
            //         gui.root()
            //             .add_textbox("textbox", 2, UITextBox::new((50, 100, 100, 15).into()));
            //     let mut checkbox = UICheckBox::new((80, 10, 14, 14).into(), false);
            //     checkbox.on_checked(Some("checked".into()));
            //     let c0 = gui.root().add_checkbox("c0", 0, checkbox);
            //     // ui.root().add_sprite("frame", 1, UISprite::new("frame".into(), (300, 50).into(), (0, 0, 96, 96).into()))?;
            //     gui.root()
            //         .add_label("test", 2, UILabel::new((330, 90).into(), "Hello", font));

            //     gui.root()
            //         .set_navigation(
            //             b0,
            //             Navigation {
            //                 up: None,
            //                 down: None,
            //                 left: None,
            //                 right: Some(c0),
            //             },
            //         )
            //         .unwrap();
            //     gui.root()
            //         .set_navigation(
            //             c0,
            //             Navigation {
            //                 up: None,
            //                 down: None,
            //                 left: Some(b0),
            //                 right: None,
            //             },
            //         )
            //         .unwrap();

            //     gui.root()
            //         .add_viewport("main_viewport", -1, UIViewport::new(IVec2::ZERO, viewport));

            //     gui.add_user("main").unwrap();

            //     let uie = ecs
            //         .add()
            //         .with(ui, gui)
            //         .with(
            //             ui_render_target,
            //             UIRenderTarget::Screen {
            //                 offset: (0, 0).into(),
            //             },
            //         )
            //         .build();
            // }

            // let controller = UIController::new()
            //     .with_primary(CommonAction::CLICK)
            //     .with_cancel(CommonAction::BACK)
            //     .with_cursor_motion(CommonAxis::CURSOR_MOTION_X, CommonAxis::CURSOR_MOTION_Y)
            //     .with_cursor_position(CommonAxis::CURSOR_X, CommonAxis::CURSOR_Y)
            //     .with_selection_move(
            //         CommonAction::UP,
            //         CommonAction::DOWN,
            //         CommonAction::LEFT,
            //         CommonAction::RIGHT,
            //     );

            // // Setup singleton
            // {
            //     ecs.add()
            //         .with(
            //             os,
            //             OS {
            //                 layout_active: true,
            //                 controller,
            //             },
            //         )
            //         .build();
        }
    }
}

impl ExclusiveSystem for OSInitialize {
    fn run(&self, ecs: &mut ECS, ctx: &mut Context) {
        // Setup resources
        self.setup_resources(ctx);
        // Setup scene
        self.setup_scene(ecs, ctx);

        let main_script = Resource::find(ctx, "main_script").expect("Script 'main' not found");
        let utils_script = Resource::find(ctx, "utils_script").expect("Script 'utils' not found");
        let script = expect!(ctx, Resource::read::<Script>(ctx, main_script));

        println!("Script: {:?}", script.source);
        let mut compiler = Compiler::default();
        let entry = compiler.add_module(
            "main_script",
            Module::Source {
                resource: main_script,
            },
        );
        compiler.add_module(
            "utils_script",
            Module::Source {
                resource: utils_script,
            },
        );
        if let Result::Err(e) = compiler.compile(entry, ctx) {
            println!("Error: {:?}", e);
        } else {
            println!("SUCCESS");
        }
    }
}
