use bevy::{math::Vec3Swizzles, prelude::*};

use crate::{
    image,
    layer::{Layer, LayerBundle},
};

pub enum CursorPosition {
    Inside(Vec2),
    Outside(Vec2),
}

#[derive(Resource)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub cursor_position: Result<Vec2, Vec2>,
    pub layer_id: Entity,
}

impl Canvas {
    pub fn new(width: u32, height: u32, layer_id: Entity) -> Self {
        Canvas {
            width,
            height,
            cursor_position: Err(Vec2::ZERO),
            layer_id,
        }
    }
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    pub fn global_cursor_position(&self) -> Vec2 {
        let pos = match self.cursor_position {
            Ok(pos) => pos,
            Err(pos) => pos,
        };

        (pos - self.size() * 0.5) * Vec2::new(1.0, -1.0)
    }

    pub fn cursor_on_canvas(&self) -> bool {
        self.cursor_position.is_ok()
    }

    pub fn in_bounds(&self, pos: Vec2) -> bool {
        (0.0..self.width as f32).contains(&pos.x) && (0.0..self.height as f32).contains(&pos.y)
    }
}

pub fn process_cursor_position(
    mut canvas: ResMut<Canvas>,
    window: Query<&Window>,
    camera: Query<(&Transform, &OrthographicProjection)>,
) {
    let window = window.single();

    let (
        Transform {
            translation: cam_pos,
            ..
        },
        proj,
    ) = camera.single();

    if let Some(mut mouse_pos) = window.cursor_position() {
        let offset = proj.area.max + cam_pos.xy() * Vec2::new(-1.0, 1.0);

        mouse_pos *= proj.scale;
        mouse_pos -= offset;

        let size = canvas.size();
        let pos = mouse_pos + size / 2.0;
        let (x, y, w, h) = (pos.x, pos.y, size.x, size.y);

        // println!("x: {}, y: {}", x, y);

        canvas.cursor_position = if (0.0..w).contains(&x) && (0.0..h).contains(&y) {
            Ok(pos)
        } else {
            Err(pos)
        };
    }
}

pub fn setup_canvas(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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

#[derive(Component)]
pub struct CursorPreview;

pub fn setup_cursor_preview(mut commands: Commands) {
    commands.spawn((CursorPreview, SpriteBundle::default()));
}

pub fn cursor_preview(
    mut cursor: Query<(&mut Transform, &mut Visibility), With<CursorPreview>>,
    canvas: Res<Canvas>,
) {
    let (mut trans, mut visibility) = cursor.single_mut();

    if canvas.cursor_on_canvas() {
        *visibility = Visibility::Visible;
        trans.translation = canvas.global_cursor_position().extend(100.0);
    } else {
        *visibility = Visibility::Hidden
    }
}
