#![feature(drain_filter, array_chunks)]

use bevy::prelude::*;

pub mod buffer;
pub mod camera;
pub mod canvas;
pub mod commands;
pub mod layer;
pub mod timeline;
pub mod tools;

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum OperationState {
    Painting,
    Filling,
    #[default]
    Idle,
}

pub fn image(width: u32, height: u32, color: Color) -> Image {
    use bevy::render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::ImageSampler,
    };
    Image {
        data: vec![255; (width * height) as usize]
            .into_iter()
            .map(|_| color.as_rgba_u8())
            .collect::<Vec<_>>()
            .concat(),
        sampler_descriptor: ImageSampler::nearest(),
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
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

pub trait Draw {
    fn paint(&mut self, pos: Vec2, color: Color);
    fn color_at_pos(&self, pos: Vec2) -> Color;
    fn paint_buffer(&mut self, buffer: &mut [u8]);
}

impl Draw for Image {
    /// ## Panics
    /// if the position is outside the bounds
    fn paint(&mut self, pos: Vec2, color: Color) {
        let i = pos.x as u32 + pos.y as u32 * self.size().x as u32;
        let i = i as usize * 4;

        let color = color.as_rgba_u8();

        self.data[i] = color[0];
        self.data[i + 1] = color[1];
        self.data[i + 2] = color[2];
        self.data[i + 3] = color[3];
    }

    fn color_at_pos(&self, pos: Vec2) -> Color {
        let i = (pos.x as usize + pos.y as usize * self.size().x as usize) * 4;
        let [r, g, b, a] = &self.data[i..(i + 4)] else { unreachable!() };
        Color::rgba_u8(*r, *g, *b, *a)
    }

    fn paint_buffer(&mut self, buffer: &mut [u8]) {
        for (i, rgba) in buffer.array_chunks_mut::<4>().enumerate() {
            
        }
    }
}

pub fn color_distance(a: Color, b: Color) -> f32 {
    (a.r() - b.r()).abs() + (a.g() - b.g()).abs() + (a.b() - b.b()).abs() + (a.a() - b.a()).abs()
}

fn lerp_u8(a: u8, b: u8, l: u8) -> u8 {
    // self + ((other - self) as $scalar * scalar).round() as $int
    let l = l as f32 * u8::MAX as f32;
    a + ((b - a) as f32 * l) as u8
}
