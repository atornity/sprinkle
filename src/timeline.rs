use std::ops::Range;

use bevy::{prelude::*, utils::HashSet};

use crate::layer::Layer;

#[derive(Component)]
pub struct Timeline {
    layers: Vec<Entity>,
    frame_offset: i32,
}

impl Timeline {
    pub fn remove_layer(&mut self, index: usize, commands: &mut Commands) -> Option<Entity> {
        assert!(self.layers.len() > index);

        let layer = self.layers.remove(index);
        commands.get_entity(layer)?.despawn_recursive();

        Some(layer)
    }

    pub fn add_layer(&mut self, commands: &mut Commands) -> Entity {
        todo!()
    }
}

#[derive(Resource)]
pub struct GlobalTimeline {
    frame: f32,
    frame_range: Range<u16>,
    playing: bool,
    /// if empty then all timelines will be played
    timelines: HashSet<Entity>,
}

pub fn timeline_layer_fix(
    mut timelines: Query<(Entity, &mut Timeline)>,
    mut layers: Query<(Entity, &mut Layer)>,
    mut global_timeline: ResMut<GlobalTimeline>,
) {
    // add a layer entity to the timeline if it doesn't already exist
    for (layer_id, layer) in &layers {
        if let Some((timeline_id, mut timeline)) =
            layer.timeline_id.and_then(|id| timelines.get_mut(id).ok())
        {
            if !timeline.layers.contains(&layer_id) {
                timeline.layers.push(layer_id);
                warn!(
                    "layer: {:?} was missing from timeline: {:?}",
                    layer_id, timeline_id
                );
            }
        }
    }

    // remove layers from the timeline that does not exist in the world
    for (timeline_id, mut timeline) in &mut timelines {
        let mut i = 0;
        while i < timeline.layers.len() {
            let layer_id = &timeline.layers[i];
            if let Ok((_, mut layer)) = layers.get_mut(*layer_id) {
                layer.timeline_id = Some(timeline_id);
                i += 1;
            } else {
                timeline.layers.remove(i);
                warn!(
                    "layer at: {:?} in timeline: {:?} does not exist",
                    i, timeline_id
                );
            }
        }
    }

    // remove timeline from global timeline if it does not exist in the world
    if !global_timeline.timelines.is_empty() {
        global_timeline.timelines.drain_filter(|id| {
            if timelines.contains(*id) {
                false
            } else {
                warn!("timeline: {:?} in global timeline does not exist.", id);
                true
            }
        });
    }
}
