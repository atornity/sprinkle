#![feature(drain_filter)]

use std::ops::Mul;

use bevy::{prelude::*, utils::HashMap};

pub mod canvas;
pub mod command;
pub mod layer;
pub mod timeline;

#[derive(Component, Default)] // TODO: implament Default manually
pub struct Layer {
    frames: HashMap<i32, Handle<Image>>,
    width: u32,
    height: u32,
}

impl Layer {
    pub fn new(width: u32, height: u32) -> Self {
        Layer {
            frames: HashMap::new(),
            width,
            height,
        }
    }

    /// * clear - the clear color
    pub fn insert_new_frame(
        &mut self,
        frame: i32,
        clear: Color,
        images: &mut Assets<Image>,
    ) -> Option<Handle<Image>> {
        if self.frames.contains_key(&frame) {
            return None;
        }
        let image = images.add(image(self.width, self.height, clear));
        self.insert_frame(frame, image)
    }

    pub fn insert_frame(&mut self, frame: i32, image: Handle<Image>) -> Option<Handle<Image>> {
        self.frames.insert(frame, Handle::clone(&image));
        Some(image)
    }

    pub fn frame(&self, frame: i32) -> Option<&Handle<Image>> {
        self.frames.get(&frame)
    }
}

#[derive(Bundle, Default)]
pub struct LayerBundle {
    layer: Layer,
    transform: Transform,
    sprite: Sprite,
    texture: Handle<Image>,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

#[derive(Resource)]
pub struct Timeline {
    layers: Vec<Entity>,
    width: u32,
    height: u32,
    frame: i32,
}

impl Timeline {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            layers: Vec::new(),
            width,
            height,
            frame: 0,
        }
    }

    pub fn layers(&self) -> &Vec<Entity> {
        &self.layers
    }

    pub fn layers_change_this_name<'a>(
        &'a self,
        layers: &'a Query<(Entity, &'a Layer)>,
    ) -> impl Iterator<Item = (Entity, &'a Layer)> {
        layers.iter().filter(|(e, _)| self.layers.contains(e))
    }

    pub fn add_new_layer(
        &mut self,
        clear: Color,
        commands: &mut Commands,
        images: &mut Assets<Image>,
    ) -> Entity {
        let mut layer = Layer::new(self.width, self.height);
        let image = layer.insert_new_frame(self.frame, clear, images).unwrap();
        self.add_layer(image, commands)
    }

    pub fn add_layer(&mut self, image: Handle<Image>, commands: &mut Commands) -> Entity {
        let mut layer = Layer::new(self.width, self.height);
        layer.insert_frame(self.frame, Handle::clone(&image));

        let entity = commands
            .spawn(LayerBundle {
                layer,
                texture: image,
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    self.layers.len() as f32 + 1.0,
                )),
                ..Default::default()
            })
            .id();
        self.layers.push(entity);
        entity
    }

    pub fn remove_layer(&mut self, layer: Entity, commands: &mut Commands) -> bool {
        if let Some(entity_commands) = commands.get_entity(layer) {
            entity_commands.despawn_recursive();
        }
        self.layers.drain_filter(|l| *l == layer).next().is_some()
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

pub trait Draw {
    fn draw(&mut self, pos: Vec2, color: Color);
    fn try_draw(&mut self, pos: Vec2, color: Color) -> bool;
}

impl Draw for Image {
    /// ## Panics
    /// if the position is outside the bounds
    fn draw(&mut self, pos: Vec2, color: Color) {
        let i = pos.x as u32 + pos.y as u32 * self.size().x as u32;

        let i = i as usize * 4;

        let color = color.as_rgba_u8();

        self.data[i] = color[0];
        self.data[i + 1] = color[1];
        self.data[i + 2] = color[2];
        self.data[i + 3] = color[3];
    }
    /// ## Returns
    /// `true` if drawing succeeded, `false` otherwise.
    fn try_draw(&mut self, pos: Vec2, color: Color) -> bool {
        if (0.0..self.size().x).contains(&pos.x) && (0.0..self.size().y).contains(&pos.y) {
            self.draw(pos, color);
            true
        } else {
            false
        }
    }
}
