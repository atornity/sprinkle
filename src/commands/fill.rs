use bevy::utils::HashSet;

use crate::{color_distance, tools::BucketState};

use super::*;

pub struct Fill {
    buffer: Vec<u8>,
    color: Color,
    speed: f32,
}

impl Fill {
    pub fn new(color: Color, speed: f32) -> Self {
        assert!(speed > 0.0, "speed must be more than 0.0");
        Fill {
            buffer: Vec::new(),
            color,
            speed,
        }
    }
}

impl CanvasOperation for Fill {
    fn name(&self) -> &'static str {
        "Fill"
    }

    // TODO: make it context aware
    fn process(&mut self, world: &mut World, _canvas_commands: &mut CanvasCommands) {
        world.resource_scope(|world, canvas: Mut<Canvas>| {
            world.resource_scope(|world, mut images: Mut<Assets<Image>>| {
                if let Ok(pos) = canvas.cursor_position {
                    let mut layers = world.query::<&Layer>();

                    let layer = layers.get(world, canvas.layer_id).unwrap();
                    let image = images.get_mut(&layer.frames[&0]).unwrap();

                    self.buffer = image.data.clone();
                    {
                        let mut next_state = world.resource_mut::<NextState<OperationState>>();
                        next_state.set(OperationState::Filling);
                    }
                    let mut bucket_state = world.resource_mut::<BucketState>();
                    bucket_state.alive_pixels = HashSet::from([pos.as_ivec2()]);
                    bucket_state.fill_in_color = image.color_at_pos(pos);
                    bucket_state.color = self.color;
                    bucket_state.speed = self.speed;
                }
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
        self.undo(world)
    }
}

pub fn canvas_fill(
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
    mut bucket_state: ResMut<BucketState>,
    mut next_state: ResMut<NextState<OperationState>>,
    time: Res<Time>,
) {
    println!("filling! {}", bucket_state.alive_pixels.len());

    let layer = layers.get(canvas.layer_id).unwrap();
    let image = images.get_mut(&layer.frames[&0]).unwrap();

    // do it more times if running slowly
    let times = time.delta().as_millis() as f32 * bucket_state.speed;

    for _ in 0..times as usize {
        let mut new_set: HashSet<IVec2> = HashSet::new();

        for pos in &bucket_state.alive_pixels {
            image.paint(pos.as_vec2(), bucket_state.color);

            let mut move_pos = |offset: IVec2| {
                let pos = *pos + offset;

                if canvas.in_bounds(pos.as_vec2()) {
                    let col = image.color_at_pos(pos.as_vec2());

                    if col == bucket_state.fill_in_color
                        && color_distance(col, bucket_state.color) > 0.01
                    {
                        new_set.insert(pos);
                    }
                }
            };

            move_pos(IVec2::new(0, -1));
            move_pos(IVec2::new(0, 1));
            move_pos(IVec2::new(1, 0));
            move_pos(IVec2::new(-1, 0));

            if bucket_state.corner_fill {
                move_pos(IVec2::new(-1, 1));
                move_pos(IVec2::new(1, 1));
                move_pos(IVec2::new(1, -1));
                move_pos(IVec2::new(-1, -1));
            }
        }

        if !new_set.is_empty() {
            bucket_state.alive_pixels = new_set
        } else {
            next_state.set(OperationState::Idle);
            break;
        }
    }
}
