use mini3d::{
    context::SystemContext,
    ecs::{procedure::Procedure, system::SystemResult},
    engine::Engine,
    event::asset::ImportAssetEvent,
    feature::{
        asset::{
            font::Font,
            input_table::{InputAction, InputAxis, InputAxisRange, InputTable},
            material::Material,
            mesh::Mesh,
            model::Model,
            script::Script,
            system_group::{SystemGroup, SystemPipeline},
            texture::Texture,
            ui_stylesheet::UIStyleSheet,
        },
        component::{
            camera::Camera,
            free_fly::FreeFly,
            hierarchy::Hierarchy,
            lifecycle::Lifecycle,
            local_to_world::LocalToWorld,
            rotator::Rotator,
            static_mesh::StaticMesh,
            transform::Transform,
            ui::{UIRenderTarget, UI},
            viewport::Viewport,
        },
    },
    glam::{IVec2, Quat, Vec3},
    math::rect::IRect,
    prng::PCG32,
    registry::{asset::Asset, component::Component, error::RegistryError},
    renderer::{SCREEN_HEIGHT, SCREEN_RESOLUTION, SCREEN_WIDTH},
    script::compiler::Compiler,
    ui::{
        self,
        controller::UIController,
        style::{UIBoxStyle, UIImageStyle, UIMargin},
        widget::{
            button::{UIButton, UIButtonStyle},
            checkbox::{UICheckBox, UICheckBoxStyle},
            label::UILabel,
            layout::Navigation,
            sprite::UISprite,
            textbox::UITextBox,
        },
    },
    uid::UID,
};

use crate::{
    asset::DefaultAsset,
    component::os::OS,
    input::{CommonAction, CommonAxis},
};

fn setup_assets(ctx: &mut SystemContext) -> SystemResult {
    ctx.asset.add_bundle(DefaultAsset::BUNDLE).unwrap();
    let default_bundle = DefaultAsset::BUNDLE.into();

    // Register default font
    ctx.asset
        .add(Font::UID, "default", default_bundle, Font::default())?;

    // Register input tables
    ctx.input.add_table(&InputTable {
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
    ctx.input.add_table(&InputTable {
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
    ctx.asset.add(
        Material::UID,
        "alfred",
        default_bundle,
        Material {
            diffuse: "alfred".into(),
        },
    )?;
    ctx.asset.add(
        Material::UID,
        "car",
        default_bundle,
        Material {
            diffuse: "car".into(),
        },
    )?;
    ctx.asset.add(
        Model::UID,
        "car",
        default_bundle,
        Model {
            mesh: "car".into(),
            materials: Vec::from(["car".into()]),
        },
    )?;
    ctx.asset.add(
        Model::UID,
        "alfred",
        default_bundle,
        Model {
            mesh: "alfred".into(),
            materials: Vec::from(["alfred".into(), "alfred".into(), "alfred".into()]),
        },
    )?;

    // Import assets
    for import in ctx.event.import_asset() {
        match import {
            ImportAssetEvent::Material(material) => {
                ctx.asset.add(
                    Material::UID,
                    &material.name,
                    default_bundle,
                    material.data.clone(),
                )?;
            }
            ImportAssetEvent::Mesh(mesh) => {
                ctx.asset
                    .add(Mesh::UID, &mesh.name, default_bundle, mesh.data.clone())?;
            }
            ImportAssetEvent::Model(model) => {
                ctx.asset
                    .add(Model::UID, &model.name, default_bundle, model.data.clone())?;
            }
            ImportAssetEvent::Script(script) => {
                ctx.asset.add(
                    Script::UID,
                    &script.name,
                    default_bundle,
                    script.data.clone(),
                )?;
            }
            ImportAssetEvent::Texture(texture) => {
                ctx.asset.add(
                    Texture::UID,
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

fn setup_world(ctx: &mut SystemContext) -> SystemResult {
    let mut world = ctx.world.active();

    {
        let e = world.create();
        world.add(e, Lifecycle::UID, Lifecycle::alive())?;
        world.add(
            e,
            Transform::UID,
            Transform {
                translation: Vec3::new(0.0, -7.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(0.5, 0.5, 0.5),
            },
        )?;
        world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
        world.add(e, Rotator::UID, Rotator { speed: 90.0 })?;
        world.add(e, StaticMesh::UID, StaticMesh::new("alfred".into()))?;
    }
    {
        let e = world.create();
        world.add(e, Lifecycle::UID, Lifecycle::alive())?;
        world.add(
            e,
            Transform::UID,
            Transform::from_translation(Vec3::new(0.0, -7.0, 9.0)),
        )?;
        world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
        world.add(e, StaticMesh::UID, StaticMesh::new("alfred".into()))?;
    }
    {
        let mut prng = PCG32::new(12345);
        for i in 0..100 {
            let e = world.create();
            world.add(e, Lifecycle::UID, Lifecycle::alive())?;
            world.add(
                e,
                Transform::UID,
                Transform::from_translation(Vec3::new(
                    ((i / 10) * 5) as f32,
                    0.0,
                    -((i % 10) * 8) as f32,
                )),
            )?;
            world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
            world.add(e, StaticMesh::UID, StaticMesh::new("car".into()))?;
            world.add(
                e,
                Rotator::UID,
                Rotator {
                    speed: -90.0 + prng.next_f32() * 90.0 * 2.0,
                },
            )?;
            let e = world.create();
            world.add(e, Lifecycle::UID, Lifecycle::alive())?;
            world.add(
                e,
                Transform::UID,
                Transform::from_translation(Vec3::new(
                    ((i / 10) * 5) as f32,
                    10.0,
                    -((i % 10) * 8) as f32,
                )),
            )?;
            world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
            world.add(e, StaticMesh::UID, StaticMesh::new("alfred".into()))?;
            world.add(
                e,
                Rotator::UID,
                Rotator {
                    speed: -90.0 + prng.next_f32() * 90.0 * 2.0,
                },
            )?;
        }
    }
    {
        let e = world.create();
        world.add(e, Lifecycle::UID, Lifecycle::alive())?;
        world.add(
            e,
            Transform::UID,
            Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
        )?;
        world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
        world.add(e, StaticMesh::UID, StaticMesh::new("car".into()))?;
        world.add(e, Rotator::UID, Rotator { speed: 30.0 })?;
    }
    {
        let e = world.create();
        world.add(e, Lifecycle::UID, Lifecycle::alive())?;
        world.add(
            e,
            Transform::UID,
            Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
        )?;
        world.add(e, LocalToWorld::UID, LocalToWorld::default())?;
        world.add(
            e,
            FreeFly::UID,
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
        )?;
        world.add(e, StaticMesh::UID, StaticMesh::new("car".into()))?;
        world.add(e, Hierarchy::UID, Hierarchy::default())?;

        let cam = world.create();
        world.add(cam, Lifecycle::UID, Lifecycle::alive())?;
        world.add(
            cam,
            Transform::UID,
            Transform::from_translation(Vec3::new(4.0, -1.0, 0.0)),
        )?;
        world.add(cam, LocalToWorld::UID, LocalToWorld::default())?;
        world.add(cam, Camera::UID, Camera::default().with_fov(90.0))?;
        world.add(cam, Hierarchy::UID, Hierarchy::default())?;

        Hierarchy::attach(e, cam, &mut world.view_mut::<Hierarchy>(Hierarchy::UID)?)?;

        let viewport = world.create();
        world.add(
            viewport,
            Viewport::UID,
            Viewport::new(SCREEN_RESOLUTION, Some(cam)),
        )?;

        let mut stylesheet = UIStyleSheet::empty();
        {
            let texture = UID::new("GUI");
            let frame0 = UIBoxStyle::Image(UIImageStyle::sliced(
                texture,
                IRect::new(2, 34, 44, 44),
                UIMargin::new(6, 6, 6, 6),
            )?);
            let button_normal = UIBoxStyle::Image(UIImageStyle::sliced(
                texture,
                IRect::new(1, 81, 14, 14),
                UIMargin::new(3, 4, 3, 3),
            )?);
            let button_pressed = UIBoxStyle::Image(UIImageStyle::sliced(
                texture,
                IRect::new(17, 81, 14, 14),
                UIMargin::new(4, 3, 3, 3),
            )?);
            let button = UIButtonStyle::new(button_normal, button_pressed, button_normal);
            let checkbox_unchecked = UIBoxStyle::Image(UIImageStyle::sliced(
                texture,
                IRect::new(81, 257, 14, 14),
                UIMargin::new(3, 4, 3, 3),
            )?);
            let checkbox_checked = UIBoxStyle::Image(UIImageStyle::sliced(
                texture,
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
            UISprite::new("alfred".into(), (50, 50).into(), (0, 0, 64, 64).into()),
        );
        let t0 = ui
            .root()
            .add_textbox("textbox", 2, UITextBox::new((50, 100, 100, 15).into()));
        let mut checkbox = UICheckBox::new((80, 10, 14, 14).into(), false);
        checkbox.on_checked(Some("checked".into()));
        let c0 = ui.root().add_checkbox("c0", 0, checkbox);
        // ui.root().add_sprite("frame", 1, UISprite::new("frame".into(), (300, 50).into(), (0, 0, 96, 96).into()))?;
        ui.root().add_label(
            "test",
            2,
            UILabel::new((330, 90).into(), "Hello", "default".into()),
        );

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

        ui.root().add_viewport(
            "main_viewport",
            -1,
            ui::widget::viewport::UIViewport::new(IVec2::ZERO, world.uid(), viewport),
        );

        ui.add_user("main")?;

        let uie = world.create();
        world.add(uie, Lifecycle::UID, Lifecycle::alive())?;
        world.add(uie, UI::UID, ui)?;
        world.add(
            uie,
            UIRenderTarget::UID,
            UIRenderTarget::Screen {
                offset: (0, 0).into(),
            },
        )?;
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
        world.add_singleton(
            OS::UID,
            OS {
                layout_active: true,
                controller,
            },
        )?;
    }

    Ok(())
}

fn setup_scheduler(ctx: &mut SystemContext) -> SystemResult {
    let pipeline = SystemPipeline::new(&[
        UID::new("rotator"),
        UID::new("transform_propagate"),
        UID::new("ui_update"),
        UID::new("ui_render"),
        UID::new("renderer"),
        UID::new("despawn_entities"),
        UID::new("free_fly"),
        UID::new("update"),
    ]);
    let mut group = SystemGroup::empty();
    group.insert(Procedure::UPDATE, pipeline, 0);
    ctx.scheduler.add_group("os", group)?;
    Ok(())
}

fn init_system(ctx: &mut SystemContext) -> SystemResult {
    setup_assets(ctx)?;
    setup_world(ctx)?;
    setup_scheduler(ctx)?;

    let script = ctx
        .asset
        .get::<Script>(Script::UID, "test".into())?
        .expect("Script not registered");
    println!("Script: {:?}", script.source);
    // let mut tokenizer = Lexer::new(&script.source);
    // while let Some(token) = tokenizer.next_token()? {
    //     println!("Token: {:?}", token);
    // }
    if let Result::Err(e) = Compiler::compile(&script.source, false) {
        // println!("Error: {:?}", e.to_string(&script.source));
    } else {
        println!("SUCCESS");
    }

    Ok(())
}

pub fn initialize_engine(engine: &mut Engine) -> Result<(), RegistryError> {
    engine.define_static_component::<OS>(OS::NAME)?;
    engine.define_static_system("update", crate::system::update::update)?;
    engine
        .define_static_system("init", init_system)
        .expect("Failed to define init system");

    engine.invoke_system("init".into());

    Ok(())
}
