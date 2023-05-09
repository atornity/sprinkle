use bevy::{prelude::*, utils::HashSet};

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Tool {
    #[default]
    Brush,
    Bucket,
    Select,
}

#[derive(Resource, Default)]
pub struct BucketState {
    pub color: Color,
    pub corner_fill: bool,
    pub speed: f32,
    pub elapsed: f32,
    pub fill_in_color: Color,
    pub alive_pixels: HashSet<IVec2>,
}

#[derive(Resource, Default)]
pub struct BrushState {
    pub data: Option<Vec<u8>>,
    pub color: Color,
    pub last_position: Vec2,
}
