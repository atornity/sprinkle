use crate::{canvas::Canvas, layer::Layer, ColorPalette, History, HistoryItem, ImagePaint};

use super::*;

#[derive(Resource, Default)]
pub struct BrushState {
    pub data: Option<Vec<u8>>,
    pub color: Color,
    pub last_position: Option<Vec2>,
}

pub fn start_painting(
    mut brush: ResMut<BrushState>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
) {
    info!("started painting!");

    let layer = layers.get(canvas.layer_id).unwrap();
    let image = images.get_mut(&layer.frames[&0]).unwrap();

    brush.data = Some(image.data.clone());
}

pub fn stop_painting(mut brush: ResMut<BrushState>, mut history: ResMut<History>) {
    info!("stopped painting!");

    history.add(HistoryItem::Painted(brush.data.take().unwrap()));
    brush.last_position = None;
}

pub fn paint(
    mut brush: ResMut<BrushState>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = layers.get(canvas.layer_id).unwrap();
    let image = images.get_mut(&layer.frames[&0]).unwrap();

    if let Ok(next_pos) = canvas.cursor_position {
        if let Some(mut pos) = brush.last_position {
            let dir = (next_pos - pos).clamp_length_max(1.0);
            let len = dir.length();

            while pos.distance_squared(next_pos) > len {
                pos += dir;
                image.paint(pos, brush.color);
            }
        }
        image.paint(next_pos, brush.color);
        brush.last_position = Some(next_pos);
    }
}
