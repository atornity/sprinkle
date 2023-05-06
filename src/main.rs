#![feature(generic_const_exprs, array_chunks)]

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    math::Vec3Swizzles,
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::{BevyDefault, ImageSampler, ImageType, TextureFormatPixelInfo},
    },
};

fn main() {
    fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
        const WIDTH: usize = 32;

        // spawn camera
        commands.spawn(Camera2dBundle {
            projection: OrthographicProjection {
                scale: 0.1,
                ..Default::default()
            },
            ..Default::default()
        });

        // create texture
        let img = image_function::<WIDTH, 16>(|x, y| [x / 32.0, y / 16.0, 0.0, 1.0]);
        let image = images.add(img);

        commands.spawn(SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(1.0)),
            texture: Handle::clone(&image),
            ..Default::default()
        });

        // spawn cursor
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_scale(Vec3::splat(1.0))
                    .with_translation(Vec3::new(0.0, 0.0, 10.0)),
                ..Default::default()
            },
            Cursor,
        ));

        commands.insert_resource(Canvas {
            image,
            cursor_position: Err(Vec2::ZERO),
            size: Vec2::new(WIDTH as f32, 16.0),
        });
    }

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

    fn move_cursor(
        window: Query<&Window>,
        camera: Query<(&Transform, &OrthographicProjection), Without<Cursor>>,
        mut cursor: Query<(&mut Transform, &mut Sprite), With<Cursor>>,
        mut canvas: ResMut<Canvas>,
    ) {
        let (mut cursor_pos, mut sprite) = cursor.single_mut();

        let window = window.single();

        let (cam_trans, proj) = camera.single();

        if let Some(mut pos) = window.cursor_position() {
            let offset = proj.area.max + cam_trans.translation.xy() * Vec2::new(-1.0, 1.0);
            pos *= proj.scale;
            pos -= offset;

            {
                let pos = pos + canvas.size / 2.0;
                let (x, y, w, h) = (pos.x, pos.y, canvas.size.x, canvas.size.y);

                let i = pos.x as u32 + pos.y as u32 * canvas.size.x as u32;

                println!("x: {}, y: {} i: {}", x, y, i);

                if (0.0..w).contains(&x) && (0.0..h).contains(&y) {
                    canvas.cursor_position = Ok(pos);
                    sprite.color = Color::WHITE;
                } else {
                    canvas.cursor_position = Err(pos);
                    sprite.color = Color::RED;
                }
            }

            pos = pos.floor() + 0.5;
            pos.y *= -1.0;

            cursor_pos.translation = pos.extend(10.0);
        }
    }

    fn paint(
        canvas: Res<Canvas>,
        input: Res<Input<MouseButton>>,
        mut images: ResMut<Assets<Image>>,
    ) {
        if input.pressed(MouseButton::Left) {
            let image = images.get_mut(&canvas.image).unwrap();

            // TODO: only draw when cursor moved
            if let Ok(pos) = canvas.cursor_position {
                let i = pos.x as u32 + pos.y as u32 * canvas.size.x as u32;
                let i = i as usize * 4;

                image.data[i] = 255;
                image.data[i + 1] = 255;
                image.data[i + 2] = 255;
            }
        }
    }

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, (zoom_camera, move_camera))
        .add_systems(
            PostUpdate,
            (apply_system_buffers, move_cursor, paint).chain(),
        )
        .run();
}

fn image_function<const WIDTH: usize, const HEIGHT: usize>(
    f: impl Fn(f32, f32) -> [f32; 4],
) -> Image
where
    [(); WIDTH * HEIGHT * 4]:,
{
    let mut buffer = [0; WIDTH * HEIGHT * 4];

    for (i, rgba) in &mut buffer.array_chunks_mut::<4>().enumerate() {
        let x = i as f32 % WIDTH as f32;
        let y = i as f32 / WIDTH as f32;

        let mut col = [0; 4];
        for (i, c) in f(x, y).iter().enumerate() {
            col[i] = (c * 255.0) as u8;
        }

        *rgba = col;
    }

    Image {
        data: Vec::from(buffer),
        sampler_descriptor: ImageSampler::nearest(),
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width: WIDTH as u32,
                height: HEIGHT as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        texture_view_descriptor: None,
    }
}

#[derive(Resource)]
struct Canvas {
    size: Vec2,
    cursor_position: Result<Vec2, Vec2>,
    image: Handle<Image>,
}

#[derive(Component)]
struct Cursor;
