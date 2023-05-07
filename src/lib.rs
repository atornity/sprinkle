#![feature(drain_filter, array_chunks)]

use bevy::prelude::*;

pub mod camera;
pub mod canvas;
pub mod commands;
pub mod layer;
pub mod timeline;

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum OperationState {
    Painting,
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
    fn get_pixel_mut(&mut self, pos: Vec2) -> &mut [u8];
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

    fn get_pixel_mut(&mut self, pos: Vec2) -> &mut [u8] {
        let i = pos.x as u32 + pos.y as u32 * self.size().x as u32;
        let i = i as usize * 4;

        &mut self.data[i..(i + 4)]
    }
}
