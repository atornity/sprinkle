use bevy::{math::Vec3Swizzles, prelude::*};

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

#[derive(Resource, Default)]
pub struct PaintTool {
    pub color: Color,
}

impl PaintTool {
    pub fn new() -> Self {
        PaintTool::default()
    }
}
