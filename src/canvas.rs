use bevy::{math::Vec3Swizzles, prelude::*};

use crate::{command::CanvasCommands, layer::Layer};

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

#[derive(Default, Clone, Copy)]
pub enum PaintMode {
    Paint,
    Erase,
    #[default]
    None,
}

#[derive(Resource, Default)]
pub struct PaintTool {
    color: Color,
    buffer: Vec<u8>,
    paint_mode: PaintMode,
}

impl PaintTool {
    pub fn new() -> Self {
        PaintTool::default()
    }
    pub fn start_painting(&mut self, color: Color) {
        self.color = color;
        self.paint_mode = PaintMode::Paint;
    }
    pub fn stop_painting(&mut self) {
        self.paint_mode = PaintMode::None;
    }
    fn paint(&mut self, position: Vec2, width: u32) {
        let i = position.x as u32 + position.y as u32 * width;

        let i = i as usize * 4;

        let color = self.color.as_rgba_u8();

        self.buffer[i] = color[0];
        self.buffer[i + 1] = color[1];
        self.buffer[i + 2] = color[2];
        self.buffer[i + 3] = color[3];
    }
}

pub fn process_painting(
    mut paint_tool: ResMut<PaintTool>,
    mut paint_mode: Local<PaintMode>,
    canvas: Res<Canvas>,
    mut canvas_commands: ResMut<CanvasCommands>,
    layers: Query<&Layer>,
    images: Res<Assets<Image>>,
) {
    if let Ok(cursor_pos) = canvas.cursor_position {
        let size = (canvas.width * canvas.height * 4) as usize;

        use PaintMode::*;
        match (*paint_mode, paint_tool.paint_mode) {
            // painting
            (Paint, Paint) => paint_tool.paint(cursor_pos, canvas.width),
            // started painting
            (None, Paint) => {
                info!("started painting");
                // let image = images.get(&canvas.)
                let layer = layers.get(canvas.layer_id).unwrap();
                let image = images.get(&layer.frames[&0]).unwrap();
                paint_tool.buffer = image.data.clone();
            }
            // stopped painting
            (Paint, None) => {
                info!("stopped painting");
                let mut buffer = Vec::new();
                std::mem::swap(&mut buffer, &mut paint_tool.buffer);

                canvas_commands.paint(buffer)
            }
            // not painting or erasing
            (None, None) => (),
            _ => unimplemented!("erasing not implemented"),
        }
        *paint_mode = paint_tool.paint_mode;
    }
}
