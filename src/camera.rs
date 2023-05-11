use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    math::Vec3Swizzles,
    prelude::*,
};

use crate::canvas::Canvas;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.1,
            ..Default::default()
        },
        ..Default::default()
    });
}

pub fn zoom_camera(
    mut camera: Query<(&mut Transform, &mut OrthographicProjection)>,
    mut mouse_scroll: EventReader<MouseWheel>,
    canvas: Res<Canvas>,
) {
    let (mut trans, mut proj) = camera.single_mut();
    for ev in mouse_scroll.iter() {
        let s = proj.scale;

        let zoom_delta = ev.y * -0.1;

        proj.scale += zoom_delta * s;
        proj.scale = proj.scale.clamp(0.01, 1.0);

        let move_delta = (trans.translation.xy() - canvas.global_cursor_position()) * zoom_delta;
        trans.translation += move_delta.extend(0.0);
    }
}

pub fn move_camera(
    mut camera: Query<(&mut Transform, &OrthographicProjection)>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    let (mut trans, proj) = camera.single_mut();

    if keyboard_input.pressed(KeyCode::Space) || mouse_input.pressed(MouseButton::Middle) {
        for ev in mouse_motion.iter() {
            let delta = ev.delta * Vec2::new(-1.0, 1.0) * proj.scale;
            trans.translation += delta.extend(0.0);
        }
    }
}
