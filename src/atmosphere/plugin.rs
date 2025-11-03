use bevy::prelude::*;
use super::systems::*;

/// Atmospheric simulation plugin
pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            initialize_neighbors,
            mark_dirty_tiles,
            process_gas_sharing,
            update_tile_visuals,
            debug_atmosphere,
        ).chain());
    }
}
