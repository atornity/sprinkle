#![feature(generic_const_exprs, array_chunks, exclusive_range_pattern)]

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use sprinkle::{
    camera::{move_camera, zoom_camera},
    canvas::{process_cursor_position, Canvas, PaintTool},
    commands::{paint::canvas_paint, process_commands, CanvasCommands},
    image,
    layer::{Layer, LayerBundle},
    OperationState,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<OperationState>()
        .init_resource::<PaintTool>()
        .init_resource::<CanvasCommands>()
        .add_systems(Startup, setup_basic)
        .add_systems(Update, (process_cursor_position, paint, erase, fill))
        .add_systems(Update, (move_camera, zoom_camera))
        .add_systems(
            Update,
            canvas_paint.run_if(in_state(OperationState::Painting)),
        )
        .add_systems(PostUpdate, (process_commands, undo).chain())
        .run();
}

fn setup_basic(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // spawn camera
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.1,
            ..Default::default()
        },
        ..Default::default()
    });

    // add image
    let image = images.add(image(16, 16, Color::rgba(0.5, 0.5, 0.5, 0.5)));

    // spawn layer
    let layer_id = commands
        .spawn(LayerBundle {
            layer: Layer::new(image.clone(), None),
            texture: image,
            ..Default::default()
        })
        .id();

    // insert canvas
    commands.insert_resource(Canvas::new(16, 16, layer_id));
}

fn paint(mut canvas_commands: ResMut<CanvasCommands>, input: Res<Input<MouseButton>>) {
    if input.just_pressed(MouseButton::Left) {
        canvas_commands.start_painting(Color::BEIGE);
    } else if input.just_released(MouseButton::Left) {
        canvas_commands.stop_painting();
    }
}

fn fill(mut canvas_commands: ResMut<CanvasCommands>, input: Res<Input<MouseButton>>) {
    if input.just_pressed(MouseButton::Middle) {
        canvas_commands.fill(Color::BEIGE);
    }
}

fn erase(mut canvas_commands: ResMut<CanvasCommands>, input: Res<Input<MouseButton>>) {
    if input.just_pressed(MouseButton::Right) {
        canvas_commands.start_painting(Color::rgba(0.0, 0.0, 0.0, 0.0));
    } else if input.just_released(MouseButton::Right) {
        canvas_commands.stop_painting();
    }
}

fn undo(mut canvas_commands: ResMut<CanvasCommands>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Comma) {
        canvas_commands.undo()
    } else if input.just_pressed(KeyCode::Period) {
        canvas_commands.redo()
    }
}
