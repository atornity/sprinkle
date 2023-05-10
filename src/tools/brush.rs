use crate::{canvas::Canvas, layer::Layer, ColorPalette, History, HistoryItem, ImagePaint};

use super::*;

#[derive(Default)]
pub enum BrushMode {
    Pixel,
    #[default]
    Line,
}

#[derive(Resource, Default)]
pub struct BrushState {
    pub buffer: Vec<u8>,
    pub data: Option<Vec<u8>>,
    pub color: Color,
    pub last_position: Option<Vec2>,
    pub mode: BrushMode,
    is_cleared: bool,
}

impl BrushState {
    pub fn is_eraser(&self) -> bool {
        self.color.as_rgba_u8()[3] == 0
    }

    pub fn is_line_mode(&self) -> bool {
        match self.mode {
            BrushMode::Pixel => false,
            BrushMode::Line => true,
        }
    }
    pub fn is_pixel_mode(&self) -> bool {
        match self.mode {
            BrushMode::Pixel => true,
            BrushMode::Line => false,
        }
    }

    pub fn clear_buffer(&mut self, canvas_width: u32, canvas_height: u32) {
        self.buffer = vec![0; (canvas_width * canvas_height * 4) as usize];
        self.is_cleared = true;
    }

    pub fn paint_to_buffer(&mut self, position: Vec2, canvas_width: u32) {
        let i: usize = (position.x as usize + position.y as usize * canvas_width as usize) * 4;
        let color = if self.is_eraser() {
            [255, 255, 255, 255]
        } else {
            self.color.as_rgba_u8()
        };

        self.buffer[i] = color[0];
        self.buffer[i + 1] = color[1];
        self.buffer[i + 2] = color[2];
        self.buffer[i + 3] = color[3];

        self.is_cleared = false;
    }

    fn make_pixel_perfect(&mut self) {
        for i in 0..self.buffer.len() / 4 {}
    }

    pub fn get_updated_buffer(&self) -> Option<Vec<u8>> {
        let mut new = Vec::with_capacity(self.buffer.len());

        for ([r_a, g_a, b_a, a_a], [r_b, g_b, b_b, a_b]) in self
            .data
            .as_ref()?
            .array_chunks::<4>()
            .zip(self.buffer.array_chunks::<4>())
        {
            if self.is_eraser() {
                if *a_b == 255 {
                    new.extend([0, 0, 0, 0]);
                } else {
                    new.extend([r_a, g_a, b_a, a_a]);
                }
            } else {
                let s = *b_b as f32 / 255.0;

                let col_v = Vec3::lerp(
                    Vec3::new(*r_a as f32, *g_a as f32, *b_a as f32),
                    Vec3::new(*r_b as f32, *g_b as f32, *b_b as f32),
                    s,
                );

                let (r, g, b) = (col_v.x as u8, col_v.y as u8, col_v.z as u8);

                let a = a_a.checked_add(*a_b).unwrap_or(u8::MAX);

                new.extend([r, g, b, a]);
            }
        }
        Some(new)
    }

    fn apply_buffer_to_layer(
        &mut self,
        canvas: &Res<Canvas>,
        layers: &Query<&Layer>,
        images: &mut ResMut<Assets<Image>>,
    ) {
        let layer = layers.get(canvas.layer_id).unwrap();
        let image = images.get_mut(&layer.frames[&0]).unwrap();
        if self.data.is_none() {
            self.data = Some(image.data.clone());
        }
        let layer = layers.get(canvas.layer_id).unwrap();
        let image = images.get_mut(&layer.frames[&0]).unwrap();
        image.data = self.get_updated_buffer().unwrap();
    }
}

pub fn start_painting(
    mut brush: ResMut<BrushState>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
) {
    info!("started painting!");

    let layer = layers.get(canvas.layer_id).unwrap();
    let image = images.get_mut(&layer.frames[&0]).unwrap();

    brush.data = Some(image.data.clone());
    brush.last_position = canvas.cursor_position.ok();
    brush.clear_buffer(canvas.width, canvas.height);
}

pub fn stop_painting(mut brush: ResMut<BrushState>, mut history: ResMut<History>) {
    info!("stopped painting!");

    history.add(HistoryItem::Painted(brush.data.take().unwrap()));
    brush.last_position = None;
}

pub fn paint(
    mut brush: ResMut<BrushState>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = layers.get(canvas.layer_id).unwrap();
    let image = images.get_mut(&layer.frames[&0]).unwrap();

    if let Ok(next_pos) = canvas.cursor_position {
        if let Some(mut pos) = brush.last_position {
            if pos.as_uvec2() == next_pos.as_uvec2() {
                return;
            }

            if brush.is_line_mode() {
                brush.clear_buffer(canvas.width, canvas.height);
            }

            let dir = (next_pos - pos).clamp_length_max(1.0);
            let len = dir.length();

            while pos.distance(next_pos) > len {
                pos += dir;
                // image.paint(pos, brush.color);
                brush.paint_to_buffer(pos, canvas.width);
            }
        }
        // image.paint(next_pos, brush.color);
        brush.paint_to_buffer(next_pos, canvas.width);

        if brush.is_pixel_mode() {
            brush.last_position = Some(next_pos);
        }

        // image.data = brush.get_updated_buffer();
        brush.apply_buffer_to_layer(&canvas, &layers, &mut images);
    } else {
        brush.last_position = None;
    }
}

pub fn brush_preview(
    mut brush: ResMut<BrushState>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Ok(pos) = canvas.cursor_position {
        if let Some(last_pos) = brush.last_position {
            if last_pos.as_uvec2() == pos.as_uvec2() {
                return;
            }
            brush.clear_buffer(canvas.width, canvas.height);
            brush.paint_to_buffer(pos, canvas.width);
            brush.apply_buffer_to_layer(&canvas, &layers, &mut images);
        }
        brush.last_position = Some(pos);
    } else {
        if !brush.is_cleared {
            brush.clear_buffer(canvas.width, canvas.height);
            brush.apply_buffer_to_layer(&canvas, &layers, &mut images);
        }
        brush.last_position = None;
    }
}
