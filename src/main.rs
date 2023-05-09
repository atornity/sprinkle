#![feature(generic_const_exprs, array_chunks, exclusive_range_pattern)]

use bevy::{math::Vec3Swizzles, prelude::*};
use sprinkle::{
    camera::{move_camera, setup_camera, zoom_camera},
    canvas::{cursor_position, cursor_preview, setup_canvas, setup_cursor_preview},
    tools::{
        brush::{paint, start_painting, stop_painting},
        BrushState, BucketState, Tool,
    },
    undo_redo, ColorPalette, ColorState, History, ToolState,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<ToolState>()
        .add_state::<Tool>()
        .init_resource::<BrushState>()
        .init_resource::<BucketState>()
        .init_resource::<History>()
        .init_resource::<ColorPalette>()
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
                cursor_position,
                background_paralax,
                undo_redo,
                change_tool,
                move_camera,
                zoom_camera,
            ),
        )
        .add_systems(PostUpdate, cursor_preview)
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
struct Background;

fn setup_background(mut commands: Commands) {
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
        Background,
    ));
}

fn background_paralax(
    mut background: Query<&mut Transform, (With<Background>, Without<Camera2d>)>,
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

fn paint_input(
    input: Res<Input<MouseButton>>,
    mut next_state: ResMut<NextState<ToolState>>,
    color: Res<ColorPalette>,
    mut brush: ResMut<BrushState>,
) {
    if input.just_pressed(MouseButton::Left) {
        brush.color = color.primary_color();
        next_state.set(ToolState::Painting);
    }
    if input.just_released(MouseButton::Left) {
        next_state.set(ToolState::Idle);
    }
    if input.just_pressed(MouseButton::Right) {
        brush.color = color.secondary_color();
        next_state.set(ToolState::Painting);
    }
    if input.just_released(MouseButton::Right) {
        next_state.set(ToolState::Idle);
    }
}
