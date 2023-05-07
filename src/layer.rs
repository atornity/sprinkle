use bevy::{prelude::*, utils::HashMap};

#[derive(Component)]
pub struct Layer {
    pub frames: HashMap<i32, Handle<Image>>,
    pub timeline_id: Option<Entity>,
}

impl Layer {
    pub fn new(image: Handle<Image>, timeline_id: Option<Entity>) -> Self {
        Self {
            frames: HashMap::from([(0, image)]),
            timeline_id,
        }
    }
}

impl Default for Layer {
    fn default() -> Self {
        Layer {
            frames: HashMap::new(),
            timeline_id: None,
        }
    }
}

#[derive(Bundle, Default)]
pub struct LayerBundle {
    pub layer: Layer,
    pub transform: Transform,
    pub sprite: Sprite,
    pub texture: Handle<Image>,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}
