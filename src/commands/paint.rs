use super::*;

pub struct Paint {
    buffer: Vec<u8>,
}

impl CanvasOperation for Paint {
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

impl Paint {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }
}

impl Default for Paint {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasCommand for Paint {
    fn process(&mut self, world: &mut World, _canvas_commands: &mut CanvasCommands) {
        world.resource_scope(|world, _paint_tool: Mut<PaintTool>| {
            world.resource_scope(|world, mut next_state: Mut<NextState<MyState>>| {
                world.resource_scope(|world, canvas: Mut<Canvas>| {
                    world.resource_scope(|world, images: Mut<Assets<Image>>| {
                        let mut layers = world.query::<&Layer>();

                        let layer = layers.get(world, canvas.layer_id).unwrap();
                        let image = images.get(&layer.frames[&0]).unwrap();

                        self.buffer = image.data.clone();
                    });
                });
                next_state.set(MyState::Painting);
            });
        });
    }

    fn name(&self) -> &'static str {
        "Paint"
    }
}

pub struct StopPaint;
impl CanvasCommand for StopPaint {
    fn process(&mut self, world: &mut World, _canvas_commands: &mut CanvasCommands) {
        world.resource_scope(|world, _paint_tool: Mut<PaintTool>| {
            world.resource_scope(|_world, mut next_state: Mut<NextState<MyState>>| {
                next_state.set(MyState::Idle);
            });
        });
    }

    fn name(&self) -> &'static str {
        "StopPaint"
    }
}

pub fn canvas_paint(
    paint_tool: ResMut<PaintTool>,
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
