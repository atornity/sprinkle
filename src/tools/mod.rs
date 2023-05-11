use bevy::{prelude::*, utils::HashSet};

use crate::{canvas::Canvas, layer::Layer};

pub mod brush;
pub mod bucket;

pub use {brush::BrushState, bucket::BucketState};

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Tool {
    #[default]
    Brush,
    Bucket,
    Select,
}

#[derive(Default)]
pub struct ToolBuffer {
    pub color: Color,
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u8>,
    pub data: Option<Vec<u8>>,
    pub is_cleared: bool,
}

impl ToolBuffer {
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
    pub fn is_transparent(&self) -> bool {
        self.color.as_rgba_u8()[3] == 0
    }

    pub fn clear_buffer(&mut self) {
        self.buffer = vec![0; (self.width * self.height * 4) as usize];
        self.is_cleared = true;
    }

    pub fn draw_point(&mut self, pos: IVec2) {
        self.is_cleared = false;

        let idx: usize = (pos.x + pos.y * self.width as i32) as usize * 4;

        let color = if self.is_transparent() {
            [255, 255, 255, 255]
        } else {
            self.color.as_rgba_u8()
        };

        self.buffer[idx] = color[0];
        self.buffer[idx + 1] = color[1];
        self.buffer[idx + 2] = color[2];
        self.buffer[idx + 3] = color[3];
    }

    pub fn get_final_image(&self) -> Option<Vec<u8>> {
        let data = self.data.as_ref()?;
        let mut new = Vec::with_capacity(self.buffer.len());

        for ([r0, g0, b0, a0], [r1, g1, b1, a1]) in data
            .array_chunks::<4>()
            .zip(self.buffer.array_chunks::<4>())
        {
            if self.is_transparent() {
                if *a1 == 255 {
                    new.extend([0, 0, 0, 0]);
                } else {
                    new.extend([r0, g0, b0, a0]);
                }
            } else {
                let s = *b1 as f32 / 255.0;

                let col_v = Vec3::lerp(
                    Vec3::new(*r0 as f32, *g0 as f32, *b0 as f32),
                    Vec3::new(*r1 as f32, *g1 as f32, *b1 as f32),
                    s,
                );

                let (r, g, b) = (col_v.x as u8, col_v.y as u8, col_v.z as u8);

                let a = a0.checked_add(*a1).unwrap_or(u8::MAX);

                new.extend([r, g, b, a]);
            }
        }
        Some(new)
    }

    pub fn clone_data_from_image(
        &mut self,
        canvas: &Res<Canvas>,
        layers: &Query<&Layer>,
        images: &mut ResMut<Assets<Image>>,
    ) {
        let layer = layers.get(canvas.layer_id).unwrap();
        let image = images.get_mut(&layer.frames[&0]).unwrap();
        self.data = Some(image.data.clone());
    }

    pub fn apply_buffer_to_layer(
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
        image.data = self.get_final_image().unwrap();
    }
}
