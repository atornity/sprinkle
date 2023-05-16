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

pub const WIDTH: u32 = 512;
pub const HEIGHT: u32 = 512;

#[derive(Resource, Default)]
pub struct ColorPalette {
    pub palette: Vec<Color>,
    pub color_state: ColorState,
}

impl ColorPalette {
    pub fn primary_color(&self) -> Color {
        match &self.color_state {
            ColorState::Indexed { primary: index, .. } => self.palette[*index as usize],
            ColorState::Color { primary: color, .. } => *color,
        }
    }
    pub fn secondary_color(&self) -> Color {
        match &self.color_state {
            ColorState::Indexed { primary: index, .. } => self.palette[*index as usize],
            ColorState::Color {
                secondary: color, ..
            } => *color,
        }
    }
    pub fn set_primary(&mut self, index: u8) {
        match &mut self.color_state {
            ColorState::Indexed { primary, .. } => *primary = index,
            ColorState::Color { primary, .. } => *primary = self.palette[index as usize],
        }
    }
}

pub enum ColorState {
    Indexed { primary: u8, secondary: u8 },
    Color { primary: Color, secondary: Color },
}

impl ColorState {
    pub fn set_primary(&mut self, color: Color) {}
}

impl Default for ColorState {
    fn default() -> Self {
        ColorState::Color {
            primary: Color::WHITE,
            secondary: Color::NONE,
        }
    }
}

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
    Filled(Vec<u8>),
    Selected(Vec<u8>),
}

pub fn undo_redo(
    mut history: ResMut<History>,
    _brush_state: ResMut<BrushState>,
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
                HistoryItem::Filled(data) => {
                    info!("undo fill");
                    std::mem::swap(data, &mut image.data);
                }
                _ => unimplemented!(),
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
                HistoryItem::Filled(data) => {
                    info!("undo fill");
                    std::mem::swap(data, &mut image.data);
                }
                _ => unimplemented!(),
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
    fn color_at_pos(&self, pos: IVec2) -> Color;
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

    fn color_at_pos(&self, pos: IVec2) -> Color {
        let i = (pos.y * self.size().x as i32 + pos.x) as usize * 4;
        let [r, g, b, a] = &self.data[i..(i + 4)] else { unreachable!() };
        Color::rgba_u8(*r, *g, *b, *a)
    }
}

pub fn color_distance(a: Color, b: Color) -> f32 {
    (a.r() - b.r()).abs() + (a.g() - b.g()).abs() + (a.b() - b.b()).abs() + (a.a() - b.a()).abs()
}

pub fn compare_color(a: [u8; 4], b: [u8; 4]) {
    todo!()
}

pub fn in_img_bounds(pos: IVec2, width: u32, height: u32) -> bool {
    pos.x >= 0 && pos.x < width as i32 && pos.y >= 0 && pos.y < height as i32
}

pub fn img_pos_to_index(pos: IVec2, width: u32) -> usize {
    (pos.y * width as i32 + pos.x) as usize * 4
}

pub fn index_to_img_pos(index: usize, width: u32) -> IVec2 {
    UVec2::new(index as u32 % width, index as u32 / width).as_ivec2()
}

pub fn color_at_img_pos(pos: IVec2, width: u32, image: &[u8]) -> Color {
    let idx = img_pos_to_index(pos, width);
    let c = &image[idx..idx + 3];
    Color::rgba_u8(c[0], c[1], c[2], c[3])
}
