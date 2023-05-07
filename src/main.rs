#![feature(generic_const_exprs, array_chunks, exclusive_range_pattern)]

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use sprinkle::{
    canvas::{process_cursor_position, Canvas, PaintTool},
    commands::{paint::canvas_paint, process_commands, CanvasCommands},
    image,
    layer::{Layer, LayerBundle},
    MyState,
};

fn main() {
    // fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    //     const WIDTH: usize = 32;
    //     const HEIGHT: usize = 32;

    //     // spawn camera
    //     commands.spawn(Camera2dBundle {
    //         projection: OrthographicProjection {
    //             scale: 0.1,
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     });

    //     // create texture
    //     let img = default_image(WIDTH, HEIGHT);

    //     let image = images.add(img);

    //     let mut timeline = Timeline::new(WIDTH as u32, HEIGHT as u32);

    //     let mut canvas = Canvas {
    //         layer_id: None,
    //         cursor_position: Err(Vec2::ZERO),
    //         layer_index: 0,
    //         size: Vec2::new(WIDTH as f32, HEIGHT as f32),
    //     };

    //     canvas.layer_id = Some(timeline.add_layer(image, &mut commands));

    //     commands.insert_resource(timeline);
    //     commands.insert_resource(canvas);

    //     // spawn cursor
    //     commands.spawn((
    //         SpriteBundle {
    //             transform: Transform::from_scale(Vec3::splat(1.0))
    //                 .with_translation(Vec3::new(0.0, 0.0, 100.0)),
    //             ..Default::default()
    //         },
    //         Cursor,
    //     ));

    //     commands.spawn((
    //         SpriteBundle {
    //             sprite: Sprite {
    //                 color: Color::rgba(0.5, 0.5, 0.5, 0.5),
    //                 ..Default::default()
    //             },
    //             transform: Transform::from_scale(Vec3::new(WIDTH as f32, HEIGHT as f32, 1.0)),
    //             ..Default::default()
    //         },
    //         Background,
    //     ));
    // }

    fn zoom_camera(
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

    fn move_camera(
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

    // fn move_cursor(
    //     window: Query<&Window>,
    //     camera: Query<(&Transform, &OrthographicProjection), Without<Cursor>>,
    //     layers: Query<&Transform, With<Layer>>,
    //     mut cursor: Query<(&mut Transform, &mut Sprite), (With<Cursor>, Without<Layer>)>,
    //     mut canvas: ResMut<Canvas>,
    // ) {
    //     let (mut cursor_pos, mut sprite) = cursor.single_mut();

    //     let window = window.single();

    //     let (cam_trans, proj) = camera.single();

    //     if let Some(mut pos) = window.cursor_position() {
    //         let canvas_offset = canvas
    //             .layer_id
    //             .map(|id| layers.get(id).unwrap().translation.xy())
    //             .unwrap_or(Vec2::ZERO);

    //         let cam_offset = cam_trans.translation.xy();

    //         let offset = proj.area.max + cam_offset * Vec2::new(-1.0, 1.0);

    //         pos *= proj.scale;
    //         pos -= offset;

    //         {
    //             let pos = pos + canvas.size / 2.0 + canvas_offset * Vec2::new(-1.0, 1.0);
    //             let (x, y, w, h) = (pos.x, pos.y, canvas.size.x, canvas.size.y);

    //             // println!("x: {}, y: {}", x, y);

    //             if (0.0..w).contains(&x) && (0.0..h).contains(&y) {
    //                 canvas.cursor_position = Ok(pos - canvas_offset);
    //                 sprite.color = Color::WHITE;
    //             } else {
    //                 canvas.cursor_position = Err(pos - canvas_offset);
    //                 sprite.color = Color::RED;
    //             }
    //         }

    //         pos = pos.floor() + 0.5;
    //         pos.y *= -1.0;

    //         cursor_pos.translation = pos.extend(10.0);
    //     }
    // }

    // fn paint(
    //     canvas: Res<Canvas>,
    //     input: Res<Input<MouseButton>>,
    //     layers: Query<(&Layer, &Transform)>,
    //     mut images: ResMut<Assets<Image>>,
    // ) {
    //     if input.pressed(MouseButton::Left) {
    //         // let image = images.get_mut(&canvas.image).unwrap();
    //         let frame_index = 0;

    //         let mut offset = Vec2::ZERO; // and this

    //         // TODO: only draw when cursor moved

    //         if let Ok(pos) = canvas.cursor_position {
    //             let Some(image) = canvas
    //                 .layer_id
    //                 .and_then(|id| {
    //                     layers.get(id).ok().and_then(|(layer, trans)| {
    //                         offset = trans.translation.xy();
    //                         layer
    //                             .frame(frame_index)
    //                             .and_then(|frame| images.get_mut(frame))
    //                     })
    //                 }) else {
    //                     warn!("Something went wrong while trying to draw");
    //                     return
    //                 };

    //             image.draw(
    //                 pos + offset, // TODO: remove this
    //                 match canvas.layer_index % 4 {
    //                     0 => Color::ALICE_BLUE,
    //                     1 => Color::ANTIQUE_WHITE,
    //                     2 => Color::AQUAMARINE,
    //                     3 => Color::MAROON,
    //                     _ => unreachable!(),
    //                 },
    //             );
    //         }
    //     }
    // }

    // fn add_image(
    //     mut commands: Commands,
    //     mut canvas: ResMut<Canvas>,
    //     mut timeline: ResMut<Timeline>,
    //     mut images: ResMut<Assets<Image>>,
    //     input: Res<Input<KeyCode>>,
    // ) {
    //     if !input.just_pressed(KeyCode::N) {
    //         return;
    //     }

    //     println!("!!! addded image !!!");
    //     canvas.layer_id = Some(timeline.add_new_layer(
    //         Color::rgba(0.0, 0.0, 0.0, 0.0),
    //         &mut commands,
    //         &mut images,
    //     ));
    //     canvas.layer_index = timeline.layers().len() as i32 - 1;
    //     canvas.layer_id = Some(timeline.layers()[canvas.layer_index as usize]);
    // }

    // fn change_layer(
    //     mut canvas: ResMut<Canvas>,
    //     timeline: Res<Timeline>,
    //     input: Res<Input<KeyCode>>,
    // ) {
    //     if input.just_pressed(KeyCode::Up) {
    //         canvas.layer_index += 1;
    //     } else if input.just_pressed(KeyCode::Down) {
    //         canvas.layer_index -= 1;
    //     } else {
    //         return;
    //     }

    //     let layer_count = timeline.layers().len() as i32;

    //     match canvas.layer_index {
    //         ..0 => canvas.layer_index = layer_count - 1,
    //         1.. if layer_count != 0 => canvas.layer_index %= layer_count,
    //         _ => (),
    //     }
    //     if layer_count != 0 {
    //         canvas.layer_index %= layer_count;
    //         canvas.layer_id = Some(timeline.layers()[canvas.layer_index as usize]);
    //     }

    //     println!("changed layer to {}", canvas.layer_index);
    // }

    // fn paralax(
    //     mut query: Query<&mut Transform, (Without<Camera2d>, Without<Cursor>)>,
    //     cam: Query<&Transform, With<Camera2d>>,
    // ) {
    //     let Transform {
    //         translation: cam_pos,
    //         ..
    //     } = cam.single();

    //     query.for_each_mut(|mut trans| {
    //         let pos = trans.translation;
    //         trans.translation = (cam_pos.xy() * pos.z * 0.1).extend(pos.z)
    //     });
    // }

    // App::new()
    //     .add_plugins(DefaultPlugins)
    //     .add_systems(Startup, setup)
    //     .add_systems(Update, (add_image, change_layer, paralax))
    //     .add_systems(PreUpdate, (zoom_camera, move_camera))
    //     .add_systems(
    //         PostUpdate,
    //         (apply_system_buffers, move_cursor, paint).chain(),
    //     )
    //     .run();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<MyState>()
        .init_resource::<PaintTool>()
        .init_resource::<CanvasCommands>()
        .add_systems(Startup, setup_basic)
        .add_systems(Update, (process_cursor_position, paint))
        .add_systems(Update, (move_camera, zoom_camera))
        .add_systems(Update, canvas_paint.run_if(in_state(MyState::Painting)))
        .add_systems(PostUpdate, (process_commands, undo).chain())
        .run();
}

// #[derive(Resource)]
// struct Canvas {
//     size: Vec2,
//     cursor_position: Result<Vec2, Vec2>,
//     layer_id: Option<Entity>,
//     layer_index: i32,
//     // image: Handle<Image>,
// }

// #[derive(Component)]
// struct Cursor;

// #[derive(Component)]
// struct Background;

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

fn undo(mut canvas_commands: ResMut<CanvasCommands>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Comma) {
        canvas_commands.undo()
    } else if input.just_pressed(KeyCode::Period) {
        canvas_commands.redo()
    }
}
