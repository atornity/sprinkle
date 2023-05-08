use bevy::prelude::*;

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Tool {
    #[default]
    Brush,
    Bucket,
    Select,
}
