use super::*;

pub struct Fill {
    buffer: Vec<u8>,
    color: Color,
}

impl Fill {
    pub fn new(color: Color) -> Self {
        Fill {
            buffer: Vec::new(),
            color,
        }
    }
}

impl CanvasCommand for Fill {
    // TODO: make it context aware
    fn process(&mut self, world: &mut World, canvas_commands: &mut CanvasCommands) {
        world.resource_scope(|world, canvas: Mut<Canvas>| {
            world.resource_scope(|world, mut images: Mut<Assets<Image>>| {
                let mut layers = world.query::<&Layer>();

                let layer = layers.get(world, canvas.layer_id).unwrap();
                let image = images.get_mut(&layer.frames[&0]).unwrap();

                self.buffer = image.data.clone();

                let this_color;
                if let Ok(pos) = canvas.cursor_position {
                    this_color = image.get_pixel_mut(pos);
                } else {
                    panic!("cursor not on canvas")
                }

                for [r, g, b, a] in self.buffer.array_chunks_mut::<4>() {
                    let mut delta = this_color[0] as f32 - *r as f32;
                    delta += this_color[1] as f32 - *g as f32;
                    delta += this_color[2] as f32 - *b as f32;
                    delta += this_color[3] as f32 - *a as f32;
                    delta /= 255.0;
                    if delta < 0.1 {
                        let color = self.color.as_rgba_u8();
                        *r = color[0];
                        *g = color[1];
                        *b = color[2];
                        *a = color[3];
                    }
                }

                std::mem::swap(&mut self.buffer, &mut image.data);
            });
        });
    }

    fn name(&self) -> &'static str {
        "Fill"
    }
}
