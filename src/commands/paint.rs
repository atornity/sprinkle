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
        world.resource_scope(|world, mut paint_tool: Mut<PaintTool>| {
            world.resource_scope(|world, mut next_state: Mut<NextState<OperationState>>| {
                world.resource_scope(|world, canvas: Mut<Canvas>| {
                    world.resource_scope(|world, images: Mut<Assets<Image>>| {
                        let mut layers = world.query::<&Layer>();

                        let layer = layers.get(world, canvas.layer_id).unwrap();
                        let image = images.get(&layer.frames[&0]).unwrap();

                        self.buffer = image.data.clone();
                    });
                });
                next_state.set(OperationState::Painting);
            });
            paint_tool.color = self.color;
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
        world.resource_scope(|world, _paint_tool: Mut<PaintTool>| {
            world.resource_scope(|_world, mut next_state: Mut<NextState<OperationState>>| {
                next_state.set(OperationState::Idle);
            });
        });
    }

    fn name(&self) -> &'static str {
        "Stop Paint"
    }
}

pub fn canvas_paint(
    paint_tool: Res<PaintTool>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Ok(cursor_pos) = canvas.cursor_position {
        let layer = layers.get(canvas.layer_id).unwrap();
        let image = images.get_mut(&layer.frames[&0]).unwrap();

        image.paint(cursor_pos, paint_tool.color);
    }
}
