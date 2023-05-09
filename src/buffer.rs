use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{canvas::Canvas, image, layer::Layer};

#[derive(Resource)]
pub struct ImageBuffer {
    pub handle: Handle<Image>,
    pub changed: HashMap<UVec2, [u8; 4]>,
}

impl ImageBuffer {
    pub fn paint(
        &mut self,
        pos: UVec2,
        color: [u8; 4],
        width: u32,
        images: &mut Assets<Image>,
    ) -> bool {
        match self.changed.insert(pos, color) {
            Some(old_color) if old_color == color => return false,
            _ => (),
        }
        let i = (pos.x + pos.y * width) as usize * 4;

        let image = images.get_mut(&self.handle).unwrap();

        image.data[i] = color[0];
        image.data[i + 1] = color[1];
        image.data[i + 2] = color[2];
        image.data[i + 3] = color[3];

        true
    }

    pub fn apply_to_image(
        &mut self,
        width: u32,
        images: &mut Assets<Image>,
        handle: &Handle<Image>,
    ) {
        let image = images.get_mut(handle).unwrap();

        // update the image
        for (pos, color) in &self.changed {
            let i = (pos.x + pos.y * width) as usize * 4;

            image.data[i] = color[0];
            image.data[i + 1] = color[1];
            image.data[i + 2] = color[2];
            image.data[i + 3] = color[3];
        }
    }

    pub fn clear(&mut self, width: u32, height: u32, images: &mut Assets<Image>) {
        self.changed = HashMap::new();

        let image = images.get_mut(&self.handle).unwrap();

        image.data = vec![0; (width * height * 4) as usize]
    }
}

#[derive(Component, Default)]
pub struct BufferPreview;

#[derive(Bundle, Default)]
pub struct BufferPreviewBundle {
    pub transform: Transform,
    pub sprite: Sprite,
    pub texture: Handle<Image>,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    _buffer_preview: BufferPreview,
}

pub fn buffer_preview(
    canvas: Res<Canvas>,
    layers: Query<(&Layer, &Transform)>,
    mut images: ResMut<Assets<Image>>,
    mut buffer_preview: Query<(&Handle<Image>, &mut Transform), With<BufferPreview>>,
    buffer: Res<ImageBuffer>,
) {
    // let (layer, layer_trans) = layers.get(canvas.layer_id).unwrap();
    // let image = images.get_mut(&layer.frames[&0]).unwrap();

    // let (buffer_image_handle, mut buffer_preview_trans) = buffer_preview.single_mut();
    // buffer_preview_trans.translation.z = layer_trans.translation.z + 0.1;

    // let mut buffer_image = images.get_mut(buffer_image_handle).unwrap();
    // buffer_image.data
}

pub fn setup_buffer(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = images.add(image(128, 128, Color::rgba(0.0, 0.0, 0.0, 0.0)));

    commands.spawn(BufferPreviewBundle {
        texture: image.clone(),
        ..Default::default()
    });

    commands.insert_resource(ImageBuffer {
        handle: image,
        changed: HashMap::new(),
    });
}

pub fn buffer_position(
    canvas: Res<Canvas>,
    layers: Query<&Transform, With<Layer>>,
    mut buffer_preview: Query<&mut Transform, (With<BufferPreview>, Without<Layer>)>,
) {
    let mut bpt = buffer_preview.single_mut();
    let lt = layers.get(canvas.layer_id).unwrap();

    bpt.translation.z = lt.translation.z + 0.1;
}
