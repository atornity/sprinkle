use bevy::utils::HashSet;
use rand::{self, Rng};

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
                    bucket_state.elapsed = 0.0;
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

    // how many times we should run the loop this frame
    let times =
        time.delta().as_millis() as f32 * bucket_state.speed * (bucket_state.elapsed * 50.0 + 1.0);

    bucket_state.elapsed += time.delta_seconds();

    let mut rng = rand::thread_rng();

    // the possible directions we can move
    const DIRECTIONS: [IVec2; 4] = [
        IVec2::NEG_X, // left
        IVec2::X,     // right
        IVec2::NEG_Y, // up
        IVec2::Y,     // down
    ];

    for _ in 0..times as usize {
        'pixel: for pos in bucket_state.alive_pixels.clone() {
            image.paint(pos.as_vec2(), bucket_state.color);

            let mut directions = Vec::from(DIRECTIONS);

            // try to move in a random direction.
            // if that direcion is outside of the sprite or the color is wrong.
            // try a different random direction.
            for i in (0..4).rev() {
                let n = if i > 0 { rng.gen_range(0..i + 1) } else { 0 };

                let new_pos = pos + directions.remove(n);

                if canvas.in_bounds(new_pos.as_vec2()) {
                    let col = image.color_at_pos(new_pos.as_vec2());

                    if color_distance(col, bucket_state.fill_in_color) < 0.01
                        && color_distance(col, bucket_state.color) > 0.01
                    {
                        bucket_state.alive_pixels.insert(new_pos);
                        continue 'pixel;
                    }
                }
            }

            // kill the pixel if we failed to move in all directions
            bucket_state.alive_pixels.remove(&pos);
        }
        // transition to idle if there are no more pixels alive
        if bucket_state.alive_pixels.is_empty() {
            next_state.set(OperationState::Idle);
            break;
        }
    }
}
