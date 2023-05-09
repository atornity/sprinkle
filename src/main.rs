#![feature(generic_const_exprs, array_chunks, exclusive_range_pattern)]

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    math::Vec3Swizzles,
    prelude::*,
};
use sprinkle::{
    buffer::{buffer_position, setup_buffer},
    camera::{move_camera, setup_camera, zoom_camera},
    canvas::{cursor_position, Canvas, PaintTool},
    commands::{fill::canvas_fill, paint::canvas_paint, process_commands, CanvasCommands},
    image,
    layer::{Layer, LayerBundle},
    tools::{BrushState, BucketState, Tool},
    OperationState,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<OperationState>()
        .add_state::<Tool>()
        .init_resource::<BrushState>()
        .init_resource::<BucketState>()
        .init_resource::<PaintTool>()
        .init_resource::<CanvasCommands>()
        .add_systems(
            Startup,
            (setup_canvas, setup_camera, setup_background, setup_buffer),
        )
        .add_systems(
            Update,
            (
                cursor_position,
                buffer_position,
                shadow_paralax,
                change_tool,
                paint.run_if(in_state(Tool::Brush)),
                fill.run_if(in_state(Tool::Bucket)),
            ),
        )
        .add_systems(Update, (move_camera, zoom_camera))
        .add_systems(
            Update,
            (
                canvas_paint
                    .run_if(in_state(Tool::Brush).and_then(in_state(OperationState::Painting))),
                canvas_fill
                    .run_if(in_state(Tool::Bucket).and_then(in_state(OperationState::Filling))),
            ),
        )
        .add_systems(PostUpdate, (process_commands, undo).chain())
        .run();
}

#[derive(Component)]
struct Shadow;

fn setup_background(mut commands: Commands) {
    // shadow
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.0, 0.0, 0.0, 0.2),
                ..Default::default()
            },
            transform: Transform {
                scale: Vec3::new(128.0, 128.0, 1.0),
                translation: Vec3::new(0.0, 0.0, -2.0),
                ..Default::default()
            },
            ..Default::default()
        },
        Shadow,
    ));
}

fn shadow_paralax(
    mut background: Query<&mut Transform, (With<Shadow>, Without<Camera2d>)>,
    camera: Query<&Transform, With<Camera2d>>,
) {
    let Transform {
        translation: cam_pos,
        ..
    } = camera.single();

    let mut bg = background.single_mut();

    bg.translation = (cam_pos.xy() * -0.1).extend(0.0);
}

fn setup_canvas(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // create image
    let image = images.add(image(128, 128, Color::rgba(0.0, 0.0, 0.0, 0.0)));

    // spawn layer
    let layer_id = commands
        .spawn(LayerBundle {
            layer: Layer::new(image.clone(), None),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            texture: image,
            ..Default::default()
        })
        .id();

    // insert canvas
    commands.insert_resource(Canvas::new(128, 128, layer_id));
}

fn paint(mut canvas_commands: ResMut<CanvasCommands>, input: Res<Input<MouseButton>>) {
    if input.just_pressed(MouseButton::Left) {
        canvas_commands.start_painting(Color::BEIGE);
    } else if input.just_released(MouseButton::Left) {
        canvas_commands.stop_painting();
    }

    if input.just_pressed(MouseButton::Right) {
        canvas_commands.start_painting(Color::rgba(0.0, 0.0, 0.0, 0.0));
    } else if input.just_released(MouseButton::Right) {
        canvas_commands.stop_painting();
    }
}

fn change_tool(input: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<Tool>>) {
    if input.just_pressed(KeyCode::B) {
        next_state.set(Tool::Brush);
        info!("[STATE] : Brush");
    }
    if input.just_pressed(KeyCode::G) {
        next_state.set(Tool::Bucket);
        info!("[STATE] : Bucket");
    }
    if input.just_pressed(KeyCode::R) {
        next_state.set(Tool::Select);
        info!("[STATE] : Select");
    }
}

fn fill(mut canvas_commands: ResMut<CanvasCommands>, input: Res<Input<MouseButton>>) {
    if input.just_pressed(MouseButton::Left) {
        canvas_commands.fill(Color::BEIGE);
    }

    if input.just_pressed(MouseButton::Right) {
        canvas_commands.fill(Color::rgba_u8(0, 0, 0, 0));
    }
}

fn undo(mut canvas_commands: ResMut<CanvasCommands>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Comma) {
        canvas_commands.undo()
    } else if input.just_pressed(KeyCode::Period) {
        canvas_commands.redo()
    }
}
