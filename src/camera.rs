use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

pub fn zoom_camera(
    mut query: Query<&mut OrthographicProjection>,
    mut mouse_scroll: EventReader<MouseWheel>,
) {
    let mut proj = query.single_mut();
    for ev in mouse_scroll.iter() {
        // proj.scale += ev.y * -0.1;

        let delta = ev.y * -0.1;

        let s = proj.scale;
        proj.scale += delta * s;

        proj.scale = proj.scale.clamp(0.01, 1.0);
        println!("d: {} s: {}", ev.y, proj.scale);
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
