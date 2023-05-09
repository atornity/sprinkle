use crate::{buffer::ImageBuffer, tools::BrushState};

use super::*;

pub struct Paint {
    buffer: Vec<u8>,
    color: Color,
}

impl Paint {
    pub fn new(color: Color) -> Self {
        Self {
            buffer: Vec::new(),
            color,
        }
    }
}

impl Default for Paint {
    fn default() -> Self {
        Self::new(Color::WHITE)
    }
}

impl CanvasOperation for Paint {
    fn name(&self) -> &'static str {
        "Paint"
    }

    fn process(&mut self, world: &mut World, _canvas_commands: &mut CanvasCommands) {
        world
            .resource_mut::<NextState<OperationState>>()
            .set(OperationState::Painting);

        world.resource_mut::<BrushState>().color = self.color;

        let canvas = world.resource::<Canvas>();

        let layer_id = canvas.layer_id;
        let buffer_size = canvas.width * canvas.height * 4;

        world.resource_scope(|world, mut images: Mut<Assets<Image>>| {
            world.resource_scope(|world, buffer: Mut<ImageBuffer>| {
                let mut layers = world.query::<&Layer>();
                let layer = layers.get(world, layer_id).unwrap();
                let image = images.get(&layer.frames[&0]).unwrap();

                self.buffer = image.data.clone();

                let buffer_image = images.get_mut(&buffer.handle).unwrap();
                buffer_image.data = vec![0; buffer_size as usize];
            });
        });
    }

    fn undo(&mut self, world: &mut World) {
        world.resource_scope(|world, canvas: Mut<Canvas>| {
            world.resource_scope(|world, mut images: Mut<Assets<Image>>| {
                let mut layers = world.query::<&Layer>();

                let layer = layers.get(world, canvas.layer_id).unwrap();
                let image = images.get_mut(&layer.frames[&0]).unwrap();

                std::mem::swap(&mut self.buffer, &mut image.data);
            });
        });
    }

    fn redo(&mut self, world: &mut World) {
        world.resource_scope(|world, canvas: Mut<Canvas>| {
            world.resource_scope(|world, mut images: Mut<Assets<Image>>| {
                let mut layers = world.query::<&Layer>();

                let layer = layers.get(world, canvas.layer_id).unwrap();
                let image = images.get_mut(&layer.frames[&0]).unwrap();

                std::mem::swap(&mut self.buffer, &mut image.data);
            });
        });
    }
}

pub struct StopPaint;
impl CanvasCommand for StopPaint {
    fn process(&mut self, world: &mut World, _canvas_commands: &mut CanvasCommands) {
        world
            .resource_mut::<NextState<OperationState>>()
            .set(OperationState::Idle);

        let canvas = world.resource::<Canvas>();
        let layer_id = canvas.layer_id;
        let canvas_width = canvas.width;
        let canvas_height = canvas.height;

        world.resource_scope(|world, mut images: Mut<Assets<Image>>| {
            world.resource_scope(|world, mut buffer: Mut<ImageBuffer>| {
                let mut layers = world.query::<&Layer>();
                let layer = layers.get(world, layer_id).unwrap();

                buffer.apply_to_image(canvas_width, &mut images, &layer.frames[&0]);
                buffer.clear(canvas_width, canvas_height, &mut images);
            });
        });
    }

    fn name(&self) -> &'static str {
        "Stop Paint"
    }
}

pub fn canvas_paint(
    brush: Res<BrushState>,
    canvas: Res<Canvas>,
    // layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
    mut buffer: ResMut<ImageBuffer>,
) {
    if let Ok(pos) = canvas.cursor_position {
        if buffer.paint(
            pos.as_uvec2(),
            brush.color.as_rgba_u8(),
            canvas.width,
            &mut images,
        ) {
            info!("[PAINT] color: {:?} at: {}", brush.color.as_rgba_u8(), pos)
        }
    }
}
