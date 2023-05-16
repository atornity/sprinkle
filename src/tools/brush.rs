use crate::{canvas::Canvas, layer::Layer, ColorPalette, History, HistoryItem, ImagePaint};

use super::*;

#[derive(Default)]
pub enum BrushMode {
    #[default]
    Pixel,
    Line,
}

#[derive(Default)]
pub enum BlendMode {
    #[default]
    Replace,
    Blend,
    Glaze,
}

#[derive(Resource, Default)]
pub struct BrushState {
    pub buffer: Vec<u8>,
    data: Option<Vec<u8>>,
    pub color: Color,
    start_position: Option<Vec2>,
    last_position: Option<Vec2>,
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

    pub fn draw_point(&mut self, pos: IVec2, width: u32) {
        self.is_cleared = false;

        let idx = ((pos.x + pos.y * width as i32) * 4) as usize;

        let color = if self.is_eraser() {
            [255, 255, 255, 255]
        } else {
            self.color.as_rgba_u8()
        };

        self.buffer[idx] = color[0];
        self.buffer[idx + 1] = color[1];
        self.buffer[idx + 2] = color[2];
        self.buffer[idx + 3] = color[3];
    }

    fn draw_line(&mut self, width: u32, start: IVec2, end: IVec2) {
        self.is_cleared = false;

        let color = if self.is_eraser() {
            [255, 255, 255, 255]
        } else {
            self.color.as_rgba_u8()
        };

        let mut x = start.x;
        let mut y = start.y;

        let dx = (end.x - start.x).abs();
        let dy = -(end.y - start.y).abs();

        let sx = if start.x < end.x { 1 } else { -1 };
        let sy = if start.y < end.y { 1 } else { -1 };

        let mut err = dx + dy;
        loop {
            let idx = (y * width as i32 + x) as usize * 4;

            self.buffer[idx] = color[0];
            self.buffer[idx + 1] = color[1];
            self.buffer[idx + 2] = color[2];
            self.buffer[idx + 3] = color[3];

            if x == end.x && y == end.y {
                break;
            }
            let e2 = err * 2;
            if e2 > dy {
                err += dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_line_thickness(&mut self, width: u32, start: IVec2, end: IVec2, thickness: f32) {
        todo!()
    }

    pub fn get_updated_buffer(&self) -> Option<Vec<u8>> {
        let mut new = Vec::with_capacity(self.buffer.len());

        for ([r1, g1, b1, a1], [r2, g2, b2, a2]) in self
            .data
            .as_ref()?
            .array_chunks::<4>()
            .zip(self.buffer.array_chunks::<4>())
        {
            if self.is_eraser() {
                if *a2 == 255 {
                    new.extend([0, 0, 0, 0]);
                } else {
                    new.extend([r1, g1, b1, a1]);
                }
            } else {
                let s = *a2 as f32 / 255.0;

                let col_v = Vec3::lerp(
                    Vec3::new(*r1 as f32, *g1 as f32, *b1 as f32),
                    Vec3::new(*r2 as f32, *g2 as f32, *b2 as f32),
                    s,
                );

                let (r, g, b) = (col_v.x as u8, col_v.y as u8, col_v.z as u8);

                let a = a1.checked_add(*a2).unwrap_or(u8::MAX);

                new.extend([r, g, b, a]);
            }
        }
        Some(new)
    }

    fn clone_data_from_image(
        &mut self,
        canvas: &Res<Canvas>,
        layers: &Query<&Layer>,
        images: &mut ResMut<Assets<Image>>,
    ) {
        let layer = layers.get(canvas.layer_id).unwrap();
        let image = images.get_mut(&layer.frames[&0]).unwrap();
        self.data = Some(image.data.clone());
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
    keyborad: Res<Input<KeyCode>>,
) {
    info!("started painting!");

    // set mode
    if keyborad.pressed(KeyCode::LShift) {
        brush.mode = BrushMode::Line;
    } else {
        brush.mode = BrushMode::Pixel;
    }

    brush.clone_data_from_image(&canvas, &layers, &mut images);
    if !brush.is_cleared {
        brush.clear_buffer(canvas.width, canvas.height);
    }
    if let Ok(pos) = canvas.cursor_position {
        brush.start_position = Some(pos);
        brush.draw_point(pos.as_ivec2(), canvas.width);
        brush.apply_buffer_to_layer(&canvas, &layers, &mut images);
    } else {
        brush.start_position = None;
    }
    brush.last_position = canvas.cursor_position.ok();
}

pub fn stop_painting(mut brush: ResMut<BrushState>, mut history: ResMut<History>) {
    info!("stopped painting!");

    history.add(HistoryItem::Painted(brush.data.take().unwrap()));
    brush.last_position = None;
}

pub fn painting(
    mut brush: ResMut<BrushState>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
    keyborad: Res<Input<KeyCode>>,
    mut changed_to_pixel: Local<bool>,
    mut changed_to_line: Local<bool>,
) {
    // change mode
    if keyborad.just_pressed(KeyCode::LShift) {
        info!("switched to line mode");
        *changed_to_line = true;

        brush.clear_buffer(canvas.width, canvas.height);
        brush.apply_buffer_to_layer(&canvas, &layers, &mut images);

        brush.mode = BrushMode::Line;
    } else if keyborad.just_released(KeyCode::LShift) {
        *changed_to_pixel = true;

        brush.mode = BrushMode::Pixel;
        info!("switched to pixel mode");
    }

    if let Ok(next_pos) = canvas.cursor_position {
        if let Some(last_pos) = brush.last_position {
            if last_pos.as_uvec2() == next_pos.as_uvec2() && !*changed_to_line {
                return;
            }

            let last_pos = if brush.is_pixel_mode() {
                if *changed_to_pixel {
                    brush.clear_buffer(canvas.width, canvas.height);
                    brush.apply_buffer_to_layer(&canvas, &layers, &mut images);
                }
                last_pos
            } else {
                brush.clear_buffer(canvas.width, canvas.height);
                brush.start_position.unwrap_or(last_pos)
            };

            brush.draw_line(canvas.width, last_pos.as_ivec2(), next_pos.as_ivec2());
            brush.apply_buffer_to_layer(&canvas, &layers, &mut images);
            *changed_to_line = false;
            *changed_to_pixel = false;
        }
        brush.last_position = Some(next_pos);
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
            brush.draw_point(pos.as_ivec2(), canvas.width);
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
