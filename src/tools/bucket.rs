use std::thread;

use rand::Rng;

use crate::{in_img_bounds, History, HistoryItem, ToolState};

use super::*;

#[derive(Resource, Default)]
pub struct BucketState {
    data: Option<Vec<u8>>,
    pub fill_color: Color,
    target_color: Color,
}

impl BucketState {
    fn span_fill_step(&mut self, start: IVec2, width: u32, height: u32, image: &mut [u8]) -> bool {
        if start.x < 0 || start.x >= width as i32 || start.y < 0 || start.y >= height as i32 {
            // The point is outside the screen, terminate the recursion
            return false;
        }

        let compare_color = |a: [u8; 4], b: [u8; 4]| -> u8 {
            let mut d = a[0] as i16 - b[0] as i16;
            d += a[1] as i16 - b[1] as i16;
            d += a[2] as i16 - b[2] as i16;
            d += a[3] as i16 - b[3] as i16;
            d.unsigned_abs() as u8
        };

        let idx = (start.y * width as i32 + start.x) as usize * 4;

        let color = {
            let c = &image[idx..idx + 4];
            [c[0], c[1], c[2], c[3]]
        };

        let fill_color = self.fill_color.as_rgba_u8();
        let target_color = self.target_color.as_rgba_u8();

        if compare_color(color, target_color) > 0 || compare_color(color, fill_color) == 0 {
            // The point has already been filled or is not the target color, terminate the recursion
            return false;
        }
        // Fill the point with the fill color
        image[idx] = fill_color[0];
        image[idx + 1] = fill_color[1];
        image[idx + 2] = fill_color[2];
        image[idx + 3] = fill_color[3];

        true
    }
}

pub fn start_filling(
    mut bucket: ResMut<BucketState>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
    mut next_state: ResMut<NextState<ToolState>>,
) {
    info!("started filling!");

    if let Ok(pos) = canvas.cursor_position {
        let layer = layers.get(canvas.layer_id).unwrap();
        let image = images.get_mut(&layer.frames[&0]).unwrap();

        let idx = (pos.y as u32 * canvas.width + pos.x as u32) as usize * 4;
        let color = {
            let c = &image.data[idx..idx + 4];
            Color::rgba_u8(c[0], c[1], c[2], c[3])
        };
        bucket.target_color = color;
        bucket.data = Some(image.data.clone());

        // perform fill
        let mut queue = Vec::from([pos.as_ivec2()]);

        while let Some(pos) = queue.pop() {
            if bucket.span_fill_step(pos, canvas.width, canvas.height, &mut image.data) {
                queue.push(pos + IVec2::X);
                queue.push(pos - IVec2::X);
                queue.push(pos + IVec2::Y);
                queue.push(pos - IVec2::Y);
            }
        }
    }

    next_state.set(ToolState::Idle);
}

pub fn stop_filling(mut bucket: ResMut<BucketState>, mut history: ResMut<History>) {
    info!("stopped filling!");

    history.add(HistoryItem::Filled(bucket.data.take().unwrap()));
}

pub fn filling(
    mut bucket: ResMut<BucketState>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
    mut next_state: ResMut<NextState<ToolState>>,
    time: Res<Time>,
) {
    // todo!();
    // info!("filling...");

    // let layer = layers.get(canvas.layer_id).unwrap();
    // let image = images.get_mut(&layer.frames[&0]).unwrap();

    // let mut queue = Vec::from([bucket.start_pos]);

    // while let Some(pos) = queue.pop() {
    //     if bucket.span_fill_step(pos, canvas.width, canvas.height, &mut image.data) {
    //         queue.push(pos + IVec2::X);
    //         queue.push(pos - IVec2::X);
    //         queue.push(pos + IVec2::Y);
    //         queue.push(pos - IVec2::Y);
    //     }
    // }
}
