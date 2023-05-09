use crate::{canvas::Canvas, layer::Layer, ColorPalette, History, HistoryItem, ImagePaint};

use super::*;

#[derive(Resource, Default)]
pub struct BrushState {
    pub data: Option<Vec<u8>>,
    pub color: Color,
    pub last_position: Vec2,
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

pub fn stop_painting(mut brush_state: ResMut<BrushState>, mut history: ResMut<History>) {
    info!("stopped painting!");

    history.add(HistoryItem::Painted(brush_state.data.take().unwrap()));
}

pub fn paint(
    brush: Res<BrushState>,
    canvas: Res<Canvas>,
    layers: Query<&Layer>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = layers.get(canvas.layer_id).unwrap();
    let image = images.get_mut(&layer.frames[&0]).unwrap();

    if let Ok(pos) = canvas.cursor_position {
        image.paint(pos, brush.color);
    }
}
