#![feature(generic_const_exprs, array_chunks, exclusive_range_pattern)]

use bevy::{math::Vec3Swizzles, prelude::*};
use sprinkle::{
    camera::{move_camera, setup_camera, zoom_camera},
    canvas::{cursor_position, setup_canvas, shadow_paralax},
    tools::{
        brush::{brush_preview, painting, start_painting, stop_painting, BrushMode},
        bucket::{filling, start_filling, stop_filling},
        BrushState, BucketState, Tool,
    },
    undo_redo, ColorPalette, ColorState, History, ToolState, HEIGHT, WIDTH,
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
        .add_systems(Startup, (setup_canvas, setup_camera, setup_background))
        .add_systems(PreUpdate, cursor_position)
        .add_systems(
            Update,
            (
                shadow_paralax,
                undo_redo,
                change_tool,
                move_camera,
                zoom_camera,
            ),
        )
        .add_systems(
            Update,
            (
                brush_input.run_if(in_state(Tool::Brush)),
                painting.run_if(in_state(ToolState::Painting)),
                bucket_input.run_if(in_state(Tool::Bucket)),
                filling.run_if(in_state(ToolState::Filling)),
                // brush_preview.run_if(in_state(Tool::Brush).and_then(in_state(ToolState::Idle))),
            ),
        )
        // transitions
        .add_systems(OnEnter(ToolState::Painting), start_painting)
        .add_systems(OnExit(ToolState::Painting), stop_painting)
        .add_systems(OnEnter(ToolState::Filling), start_filling)
        .add_systems(OnExit(ToolState::Filling), stop_filling)
        .run();
}

#[derive(Component)]
pub struct Background;

fn setup_background(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.0, 0.0, 0.0, 0.2),
                ..Default::default()
            },
            transform: Transform {
                scale: Vec3::new(WIDTH as f32, HEIGHT as f32, 1.0),
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        },
        Background,
    ));
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

fn brush_input(
    mouse: Res<Input<MouseButton>>,
    keyborad: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<ToolState>>,
    color: Res<ColorPalette>,
    mut brush: ResMut<BrushState>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        brush.color = color.primary_color();
        next_state.set(ToolState::Painting);
    }
    if mouse.just_released(MouseButton::Left) {
        next_state.set(ToolState::Idle);
    }

    if mouse.just_pressed(MouseButton::Right) {
        brush.color = color.secondary_color();
        next_state.set(ToolState::Painting);
    }
    if mouse.just_released(MouseButton::Right) {
        next_state.set(ToolState::Idle);
    }
}

fn bucket_input(
    input: Res<Input<MouseButton>>,
    mut next_state: ResMut<NextState<ToolState>>,
    color: Res<ColorPalette>,
    mut bucket: ResMut<BucketState>,
) {
    if input.just_pressed(MouseButton::Left) {
        bucket.fill_color = color.primary_color();
        next_state.set(ToolState::Filling);
    }
    if input.just_pressed(MouseButton::Right) {
        bucket.fill_color = color.secondary_color();
        next_state.set(ToolState::Filling);
    }
}
