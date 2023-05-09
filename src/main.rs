#![feature(generic_const_exprs, array_chunks, exclusive_range_pattern)]

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    math::Vec3Swizzles,
    prelude::*,
};
use sprinkle::{
    camera::{move_camera, setup_camera, zoom_camera},
    canvas::{cursor_preview, process_cursor_position, setup_canvas, setup_cursor_preview, Canvas},
    image,
    layer::{Layer, LayerBundle},
    tools::{BrushState, BucketState, Tool},
    undo_redo, History, HistoryItem, ImagePaint, ToolState,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<ToolState>()
        .add_state::<Tool>()
        .init_resource::<BrushState>()
        .init_resource::<BucketState>()
        .init_resource::<History>()
        .add_systems(
            Startup,
            (
                setup_canvas,
                setup_camera,
                setup_background,
                setup_cursor_preview,
            ),
        )
        .add_systems(
            Update,
            (
                process_cursor_position,
                // shadow_paralax,
                undo_redo,
                change_tool,
                cursor_preview,
            ),
        )
        .add_systems(Update, (move_camera, zoom_camera))
        .add_systems(
            Update,
            (
                paint_input.run_if(in_state(Tool::Brush)),
                paint.run_if(in_state(ToolState::Painting)),
            ),
        )
        .add_systems(OnEnter(ToolState::Painting), start_painting)
        .add_systems(OnExit(ToolState::Painting), stop_painting)
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
                translation: Vec3::new(0.0, 0.0, 0.0),
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

fn change_tool(input: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<Tool>>) {
    if input.just_pressed(KeyCode::B) {
        next_state.set(Tool::Brush);
        info!("[TOOL] : Brush");
    }
    if input.just_pressed(KeyCode::G) {
        next_state.set(Tool::Bucket);
        info!("[TOOL] : Bucket");
    }
    if input.just_pressed(KeyCode::R) {
        next_state.set(Tool::Select);
        info!("[TOOL] : Select");
    }
}

fn paint_input(input: Res<Input<MouseButton>>, mut next_state: ResMut<NextState<ToolState>>) {
    if input.just_pressed(MouseButton::Left) {
        next_state.set(ToolState::Painting);
    }
    if input.just_released(MouseButton::Left) {
        next_state.set(ToolState::Idle);
    }
}

fn start_painting(
    mut brush_state: ResMut<BrushState>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
) {
    info!("started painting!");

    let layer = layers.get(canvas.layer_id).unwrap();
    let image = images.get_mut(&layer.frames[&0]).unwrap();

    brush_state.data = Some(image.data.clone());
}

fn stop_painting(mut brush_state: ResMut<BrushState>, mut history: ResMut<History>) {
    info!("stopped painting!");

    history.add(HistoryItem::Painted(brush_state.data.take().unwrap()));
}

fn paint(
    mut brush: ResMut<BrushState>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = layers.get(canvas.layer_id).unwrap();
    let image = images.get_mut(&layer.frames[&0]).unwrap();

    if let Ok(pos) = canvas.cursor_position {
        image.paint(pos, brush.color);
    }
}
