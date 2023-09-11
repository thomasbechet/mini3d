use mini3d::{
    asset::handle::StaticAsset,
    ecs::{
        api::{ecs::ExclusiveECS, ExclusiveAPI},
        instance::SystemResult,
    },
    feature::component::{
        common::{free_fly::FreeFly, lifecycle::Lifecycle, rotator::Rotator, script::Script},
        input::input_table::{InputAction, InputAxis, InputAxisRange, InputTable},
        renderer::{
            camera::Camera, font::Font, material::Material, mesh::Mesh, model::Model,
            static_mesh::StaticMesh, texture::Texture, viewport::Viewport,
        },
        scene::{hierarchy::Hierarchy, local_to_world::LocalToWorld, transform::Transform},
        ui::{
            ui::{UIRenderTarget, UI},
            ui_stylesheet::UIStyleSheet,
        },
    },
    glam::{IVec2, Quat, Vec3},
    math::rect::IRect,
    registry::{component::StaticComponent, system::ExclusiveSystem},
    renderer::{SCREEN_HEIGHT, SCREEN_RESOLUTION, SCREEN_WIDTH},
    script::{compiler::Compiler, module::Module},
    system::event::ImportAssetEvent,
    ui::{
        controller::UIController,
        style::{UIBoxStyle, UIImageStyle, UIMargin},
        widget::{
            button::{UIButton, UIButtonStyle},
            checkbox::{UICheckBox, UICheckBoxStyle},
            label::UILabel,
            layout::Navigation,
            sprite::UISprite,
            textbox::UITextBox,
            viewport::UIViewport,
        },
    },
    utils::prng::PCG32,
};

use crate::{
    asset::DefaultAsset,
    component::os::OS,
    input::{CommonAction, CommonAxis},
};

#[derive(Default)]
pub(crate) struct OSInitialize {
    font: StaticComponent<Font>,
    material: StaticComponent<Material>,
    texture: StaticComponent<Texture>,
    mesh: StaticComponent<Mesh>,
    model: StaticComponent<Model>,
    lifecycle: StaticComponent<Lifecycle>,
    transform: StaticComponent<Transform>,
    local_to_world: StaticComponent<LocalToWorld>,
    static_mesh: StaticComponent<StaticMesh>,
    rotator: StaticComponent<Rotator>,
    free_fly: StaticComponent<FreeFly>,
    hierarchy: StaticComponent<Hierarchy>,
    camera: StaticComponent<Camera>,
    viewport: StaticComponent<Viewport>,
    ui: StaticComponent<UI>,
    ui_render_target: StaticComponent<UIRenderTarget>,
    script: StaticComponent<Script>,
    os: StaticComponent<OS>,
}

impl OSInitialize {
    pub const NAME: &'static str = "os_initialize";
}

impl OSInitialize {
    fn setup_assets(&self, api: &mut ExclusiveAPI) -> SystemResult {
        let default_bundle = api.asset.add_bundle(DefaultAsset::BUNDLE).unwrap();

        // Register default font
        api.asset
            .add(self.font, "default", default_bundle, Font::default())?;

        // Register input tables
        api.input.add_table(&InputTable {
            name: "common".to_string(),
            display_name: "Common Inputs".to_string(),
            description: "".to_string(),
            actions: Vec::from([
                InputAction {
                    name: CommonAction::CLICK.to_string(),
                    display_name: "Click".to_string(),
                    description: "UI interaction layout (click).".to_string(),
                    default_pressed: false,
                },
                InputAction {
                    name: CommonAction::BACK.to_string(),
                    display_name: "Back".to_string(),
                    description: "UI interaction layout (back).".to_string(),
                    default_pressed: false,
                },
                InputAction {
                    name: CommonAction::UP.to_string(),
                    display_name: "Up".to_string(),
                    description: "UI interaction layout (go up).".to_string(),
                    default_pressed: false,
                },
                InputAction {
                    name: CommonAction::LEFT.to_string(),
                    display_name: "Left".to_string(),
                    description: "UI interaction layout (go left).".to_string(),
                    default_pressed: false,
                },
                InputAction {
                    name: CommonAction::DOWN.to_string(),
                    display_name: "Down".to_string(),
                    description: "UI interaction layout (go down).".to_string(),
                    default_pressed: false,
                },
                InputAction {
                    name: CommonAction::RIGHT.to_string(),
                    display_name: "Right".to_string(),
                    description: "UI interaction layout (go right).".to_string(),
                    default_pressed: false,
                },
                InputAction {
                    name: CommonAction::CHANGE_CONTROL_MODE.to_string(),
                    display_name: "Change Control Mode".to_string(),
                    description: "Switch between selection and cursor control mode.".to_string(),
                    default_pressed: false,
                },
                InputAction {
                    name: CommonAction::TOGGLE_PROFILER.to_string(),
                    display_name: "Toggle Profiler".to_string(),
                    description: "Show or hide the profiler.".to_string(),
                    default_pressed: false,
                },
            ]),
            axis: Vec::from([
                InputAxis {
                    name: CommonAxis::CURSOR_X.to_string(),
                    display_name: "Cursor X".to_string(),
                    description: "Horizontal position of the mouse cursor relative to the screen."
                        .to_string(),
                    range: InputAxisRange::Clamped {
                        min: 0.0,
                        max: SCREEN_WIDTH as f32,
                    },
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::CURSOR_Y.to_string(),
                    display_name: "Cursor Y".to_string(),
                    description: "Vertical position of the mouse cursor relative to the screen."
                        .to_string(),
                    range: InputAxisRange::Clamped {
                        min: 0.0,
                        max: SCREEN_HEIGHT as f32,
                    },
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::SCROLL_MOTION.to_string(),
                    display_name: "Scroll Motion".to_string(),
                    description: "Delta scrolling value.".to_string(),
                    range: InputAxisRange::Infinite,
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::CURSOR_MOTION_X.to_string(),
                    display_name: "Cursor Motion X".to_string(),
                    description: "Delta mouvement of the mouse on the horizontal axis.".to_string(),
                    range: InputAxisRange::Infinite,
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::CURSOR_MOTION_Y.to_string(),
                    display_name: "Cursor Motion Y".to_string(),
                    description: "Delta mouvement of the mouse on the vertical axis.".to_string(),
                    range: InputAxisRange::Infinite,
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::VIEW_X.to_string(),
                    display_name: "View X".to_string(),
                    description: "View horizontal delta movement.".to_string(),
                    range: InputAxisRange::Infinite,
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::VIEW_Y.to_string(),
                    display_name: "View Y".to_string(),
                    description: "View vertical delta movement.".to_string(),
                    range: InputAxisRange::Infinite,
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::MOVE_FORWARD.to_string(),
                    display_name: "Move Forward".to_string(),
                    description: "".to_string(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::MOVE_BACKWARD.to_string(),
                    display_name: "Move Backward".to_string(),
                    description: "".to_string(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::MOVE_LEFT.to_string(),
                    display_name: "Move Left".to_string(),
                    description: "".to_string(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::MOVE_RIGHT.to_string(),
                    display_name: "Move Right".to_string(),
                    description: "".to_string(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::MOVE_UP.to_string(),
                    display_name: "Move Up".to_string(),
                    description: "".to_string(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                },
                InputAxis {
                    name: CommonAxis::MOVE_DOWN.to_string(),
                    display_name: "Move Down".to_string(),
                    description: "".to_string(),
                    range: InputAxisRange::Clamped { min: 0.0, max: 1.0 },
                    default_value: 0.0,
                },
            ]),
        })?;
        api.input.add_table(&InputTable {
            name: "default".to_string(),
            display_name: "Default Inputs".to_string(),
            description: "".to_string(),
            actions: Vec::from([
                InputAction {
                    name: "roll_left".to_string(),
                    display_name: "Roll Left".to_string(),
                    description: "".to_string(),
                    default_pressed: false,
                },
                InputAction {
                    name: "roll_right".to_string(),
                    display_name: "Roll Right".to_string(),
                    description: "".to_string(),
                    default_pressed: false,
                },
                InputAction {
                    name: "switch_mode".to_string(),
                    display_name: "Switch Mode".to_string(),
                    description: "".to_string(),
                    default_pressed: false,
                },
                InputAction {
                    name: "move_fast".to_string(),
                    display_name: "Move Fast".to_string(),
                    description: "".to_string(),
                    default_pressed: false,
                },
                InputAction {
                    name: "move_slow".to_string(),
                    display_name: "Move Slow".to_string(),
                    description: "".to_string(),
                    default_pressed: false,
                },
            ]),
            axis: Vec::from([]),
        })?;

        // Non default assets
        let alfred_texture = api.asset.find("alfred_tex").unwrap();
        let alfred_mesh = api.asset.find("alfred_mesh").unwrap();
        let car_texture = api.asset.find("car_tex").unwrap();
        let car_mesh = api.asset.find("car_mesh").unwrap();
        let alfred_material = api.asset.add(
            self.material,
            "alfred",
            default_bundle,
            Material {
                diffuse: alfred_texture,
            },
        )?;
        let car_material = api.asset.add(
            self.material,
            "car",
            default_bundle,
            Material {
                diffuse: car_texture,
            },
        )?;
        api.asset.add(
            self.model,
            "car",
            default_bundle,
            Model {
                mesh: car_mesh,
                materials: Vec::from([car_material]),
            },
        )?;
        api.asset.add(
            self.model,
            "alfred",
            default_bundle,
            Model {
                mesh: alfred_mesh,
                materials: Vec::from([alfred_material, alfred_material, alfred_material]),
            },
        )?;

        // Import assets
        while let Some(import) = api.system.poll_import() {
            match import {
                ImportAssetEvent::Material(material) => {
                    api.asset.add(
                        self.material,
                        &material.name,
                        default_bundle,
                        material.data.clone(),
                    )?;
                }
                ImportAssetEvent::Mesh(mesh) => {
                    api.asset
                        .add(self.mesh, &mesh.name, default_bundle, mesh.data.clone())?;
                }
                ImportAssetEvent::Model(model) => {
                    api.asset
                        .add(self.model, &model.name, default_bundle, model.data.clone())?;
                }
                ImportAssetEvent::Script(script) => {
                    api.asset.add(
                        self.script,
                        &script.name,
                        default_bundle,
                        script.data.clone(),
                    )?;
                }
                ImportAssetEvent::Texture(texture) => {
                    api.asset.add(
                        self.texture,
                        &texture.name,
                        default_bundle,
                        texture.data.clone(),
                    )?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn setup_scene(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) -> SystemResult {
        let alfred_model = api.asset.find("alfred").unwrap();
        let alfred_texture = api.asset.find("alfred_tex").unwrap();
        let car_model = api.asset.find("car").unwrap();
        let gui_texture = api.asset.find("GUI").unwrap();
        let font = api.asset.find("default").unwrap();
        {
            let e = ecs
                .add()
                .with(self.lifecycle, Lifecycle::alive())
                .with(
                    self.transform,
                    Transform {
                        translation: Vec3::new(0.0, -7.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(0.5, 0.5, 0.5),
                    },
                )
                .with_default(self.local_to_world)
                .with(self.rotator, Rotator { speed: 90.0 })
                .with(self.static_mesh, StaticMesh::new(alfred_model))
                .build();
        }
        {
            let e = ecs
                .add()
                .with(self.lifecycle, Lifecycle::alive())
                .with(
                    self.transform,
                    Transform::from_translation(Vec3::new(0.0, -7.0, 9.0)),
                )
                .with_default(self.local_to_world)
                .with(self.static_mesh, StaticMesh::new(alfred_model))
                .build();
        }
        {
            let mut prng = PCG32::new(12345);
            for i in 0..100 {
                ecs.add()
                    .with(self.lifecycle, Lifecycle::alive())
                    .with(
                        self.transform,
                        Transform::from_translation(Vec3::new(
                            ((i / 10) * 5) as f32,
                            0.0,
                            -((i % 10) * 8) as f32,
                        )),
                    )
                    .with_default(self.local_to_world)
                    .with(self.static_mesh, StaticMesh::new(car_model))
                    .with(
                        self.rotator,
                        Rotator {
                            speed: -90.0 + prng.next_f32() * 90.0 * 2.0,
                        },
                    )
                    .build();

                ecs.add()
                    .with(self.lifecycle, Lifecycle::alive())
                    .with(
                        self.transform,
                        Transform::from_translation(Vec3::new(
                            ((i / 10) * 5) as f32,
                            10.0,
                            -((i % 10) * 8) as f32,
                        )),
                    )
                    .with_default(self.local_to_world)
                    .with(self.static_mesh, StaticMesh::new(alfred_model))
                    .with(
                        self.rotator,
                        Rotator {
                            speed: -90.0 + prng.next_f32() * 90.0 * 2.0,
                        },
                    )
                    .build();
            }
        }
        {
            ecs.add()
                .with(self.lifecycle, Lifecycle::alive())
                .with(
                    self.transform,
                    Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
                )
                .with_default(self.local_to_world)
                .with(self.static_mesh, StaticMesh::new(car_model))
                .with(self.rotator, Rotator { speed: 30.0 })
                .build();
        }
        {
            let e = ecs
                .add()
                .with(self.lifecycle, Lifecycle::alive())
                .with(
                    self.transform,
                    Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
                )
                .with_default(self.local_to_world)
                .with(
                    self.free_fly,
                    FreeFly {
                        active: true,
                        switch_mode: "switch_mode".into(),
                        roll_left: "roll_left".into(),
                        roll_right: "roll_right".into(),
                        view_x: CommonAxis::VIEW_X.into(),
                        view_y: CommonAxis::VIEW_Y.into(),
                        move_forward: CommonAxis::MOVE_FORWARD.into(),
                        move_backward: CommonAxis::MOVE_BACKWARD.into(),
                        move_up: CommonAxis::MOVE_UP.into(),
                        move_down: CommonAxis::MOVE_DOWN.into(),
                        move_left: CommonAxis::MOVE_LEFT.into(),
                        move_right: CommonAxis::MOVE_RIGHT.into(),
                        move_fast: "move_fast".into(),
                        move_slow: "move_slow".into(),
                        free_mode: false,
                        yaw: 0.0,
                        pitch: 0.0,
                    },
                )
                .with(self.static_mesh, StaticMesh::new(car_model))
                .with_default(self.hierarchy)
                .build();

            let cam = ecs
                .add()
                .with(self.lifecycle, Lifecycle::alive())
                .with(
                    self.transform,
                    Transform::from_translation(Vec3::new(0.0, -1.0, 0.0)),
                )
                .with_default(self.local_to_world)
                .with(self.camera, Camera::default().with_fov(90.0))
                .with_default(self.hierarchy)
                .build();

            Hierarchy::attach(e, cam, &mut ecs.view_mut(self.hierarchy)?)?;

            let viewport = ecs
                .add()
                .with(self.viewport, Viewport::new(SCREEN_RESOLUTION, Some(cam)))
                .build();

            let mut stylesheet = UIStyleSheet::empty();
            {
                let frame0 = UIBoxStyle::Image(UIImageStyle::sliced(
                    gui_texture,
                    IRect::new(2, 34, 44, 44),
                    UIMargin::new(6, 6, 6, 6),
                )?);
                let button_normal = UIBoxStyle::Image(UIImageStyle::sliced(
                    gui_texture,
                    IRect::new(1, 81, 14, 14),
                    UIMargin::new(3, 4, 3, 3),
                )?);
                let button_pressed = UIBoxStyle::Image(UIImageStyle::sliced(
                    gui_texture,
                    IRect::new(17, 81, 14, 14),
                    UIMargin::new(4, 3, 3, 3),
                )?);
                let button = UIButtonStyle::new(button_normal, button_pressed, button_normal);
                let checkbox_unchecked = UIBoxStyle::Image(UIImageStyle::sliced(
                    gui_texture,
                    IRect::new(81, 257, 14, 14),
                    UIMargin::new(3, 4, 3, 3),
                )?);
                let checkbox_checked = UIBoxStyle::Image(UIImageStyle::sliced(
                    gui_texture,
                    IRect::new(97, 257, 14, 14),
                    UIMargin::new(4, 3, 3, 3),
                )?);
                let checkbox = UICheckBoxStyle::new(checkbox_unchecked, checkbox_checked);
                stylesheet.add_button_style(UIButtonStyle::DEFAULT, button)?;
                stylesheet.add_checkbox_style(UICheckBoxStyle::DEFAULT, checkbox)?;
            }
            let mut ui = UI::new(SCREEN_RESOLUTION, stylesheet);
            // let box_style = UIBoxStyle::sliced("frame".into(), (0, 0, 96, 96).into(), UIMargin::new(5, 5, 5, 5), TextureWrapMode::Repeat);
            // let button_style = UIButtonStyle::new(box_style, box_style, box_style);
            let mut button = UIButton::new(IRect::new(10, 10, 60, 20));
            button.on_pressed(Some("HELLO".into()));
            let b0 = ui.root().add_button("b0", 5, button);
            // let b1 = ui.root().add_button("b1", 0, UIButton::new(IRect::new(10, 50, 50, 20)))?;
            ui.root().add_sprite(
                "alfred",
                1,
                UISprite::new(alfred_texture, (50, 50).into(), (0, 0, 64, 64).into()),
            );
            let t0 = ui
                .root()
                .add_textbox("textbox", 2, UITextBox::new((50, 100, 100, 15).into()));
            let mut checkbox = UICheckBox::new((80, 10, 14, 14).into(), false);
            checkbox.on_checked(Some("checked".into()));
            let c0 = ui.root().add_checkbox("c0", 0, checkbox);
            // ui.root().add_sprite("frame", 1, UISprite::new("frame".into(), (300, 50).into(), (0, 0, 96, 96).into()))?;
            ui.root()
                .add_label("test", 2, UILabel::new((330, 90).into(), "Hello", font));

            ui.root().set_navigation(
                b0,
                Navigation {
                    up: None,
                    down: None,
                    left: None,
                    right: Some(c0),
                },
            )?;
            ui.root().set_navigation(
                c0,
                Navigation {
                    up: None,
                    down: None,
                    left: Some(b0),
                    right: None,
                },
            )?;

            ui.root()
                .add_viewport("main_viewport", -1, UIViewport::new(IVec2::ZERO, viewport));

            ui.add_user("main")?;

            let uie = ecs
                .add()
                .with(self.lifecycle, Lifecycle::alive())
                .with(self.ui, ui)
                .with(
                    self.ui_render_target,
                    UIRenderTarget::Screen {
                        offset: (0, 0).into(),
                    },
                )
                .build();
        }

        let controller = UIController::new()
            .with_primary(CommonAction::CLICK.into())
            .with_cancel(CommonAction::BACK.into())
            .with_cursor_motion(
                CommonAxis::CURSOR_MOTION_X.into(),
                CommonAxis::CURSOR_MOTION_Y.into(),
            )
            .with_cursor_position(CommonAxis::CURSOR_X.into(), CommonAxis::CURSOR_Y.into())
            .with_selection_move(
                CommonAction::UP.into(),
                CommonAction::DOWN.into(),
                CommonAction::LEFT.into(),
                CommonAction::RIGHT.into(),
            );

        // Setup singleton
        {
            ecs.add()
                .with(
                    self.os,
                    OS {
                        layout_active: true,
                        controller,
                    },
                )
                .build();
        }

        Ok(())
    }
}

impl ExclusiveSystem for OSInitialize {
    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) -> SystemResult {
        self.setup_assets(api)?;
        self.setup_scene(ecs, api)?;

        let main_script: StaticAsset<Script> =
            api.asset.find("main").expect("Script 'main' not found");
        let utils_script: StaticAsset<Script> =
            api.asset.find("utils").expect("Script 'utils' not found");
        let script = api.asset.read(main_script)?;

        println!("Script: {:?}", script.source);
        let mut compiler = Compiler::default();
        let entry = compiler.add_module("main".into(), Module::Source { asset: main_script });
        compiler.add_module(
            "utils".into(),
            Module::Source {
                asset: utils_script,
            },
        );
        if let Result::Err(e) = compiler.compile(entry, &api.asset) {
            println!("Error: {:?}", e);
        } else {
            println!("SUCCESS");
        }

        Ok(())
    }
}
