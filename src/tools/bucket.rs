use super::*;

#[derive(Resource, Default)]
pub struct BucketState {
    pub color: Color,
    pub corner_fill: bool,
    pub speed: f32,
    pub elapsed: f32,
    pub fill_in_color: Color,
    pub alive_pixels: HashSet<IVec2>,
}
