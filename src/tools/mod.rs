use bevy::{prelude::*, utils::HashSet};

pub mod brush;
pub mod bucket;

pub use {brush::BrushState, bucket::BucketState};

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Tool {
    #[default]
    Brush,
    Bucket,
    Select,
}
