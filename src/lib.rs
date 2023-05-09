#![feature(drain_filter, array_chunks)]

use bevy::prelude::*;
use canvas::Canvas;
use layer::Layer;
use tools::BrushState;

pub mod camera;
pub mod canvas;
pub mod layer;
pub mod timeline;
pub mod tools;

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ToolState {
    Painting,
    Filling,
    #[default]
    Idle,
}

#[derive(Resource, Default)]
pub struct History {
    pub past: Vec<HistoryItem>,
    pub future: Vec<HistoryItem>,
    max_size: usize,
}

impl History {
    pub fn add(&mut self, item: HistoryItem) {
        self.past.push(item);
        self.future.clear();
    }
}

pub enum HistoryItem {
    Painted(Vec<u8>),
}

pub fn undo_redo(
    mut history: ResMut<History>,
    mut brush_state: ResMut<BrushState>,
    input: Res<Input<KeyCode>>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = layers.get(canvas.layer_id).unwrap();
    let image = images.get_mut(&layer.frames[&0]).unwrap();

    if input.just_pressed(KeyCode::Comma) {
        if let Some(mut item) = history.past.pop() {
            match &mut item {
                HistoryItem::Painted(data) => {
                    info!("undo paint");
                    std::mem::swap(data, &mut image.data);
                }
            }
            history.future.push(item);
        }
    }

    if input.just_pressed(KeyCode::Period) {
        if let Some(mut item) = history.future.pop() {
            match &mut item {
                HistoryItem::Painted(data) => {
                    info!("redo paint");
                    std::mem::swap(data, &mut image.data);
                }
            }
            history.past.push(item);
        }
    }
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

pub trait ImagePaint {
    fn paint(&mut self, pos: Vec2, color: Color);
    fn get_pixel_mut(&mut self, pos: Vec2) -> &mut [u8]; // TODO: remove this
    fn color_at_pos(&self, pos: Vec2) -> Color;
}

impl ImagePaint for Image {
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

    fn color_at_pos(&self, pos: Vec2) -> Color {
        let i = (pos.x as usize + pos.y as usize * self.size().x as usize) * 4;
        let [r, g, b, a] = &self.data[i..(i + 4)] else { unreachable!() };
        Color::rgba_u8(*r, *g, *b, *a)
    }
}

pub fn color_distance(a: Color, b: Color) -> f32 {
    (a.r() - b.r()).abs() + (a.g() - b.g()).abs() + (a.b() - b.b()).abs() + (a.a() - b.a()).abs()
}
