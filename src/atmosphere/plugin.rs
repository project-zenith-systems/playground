use bevy::prelude::*;
use super::systems::*;

/// Atmospheric simulation plugin
pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            initialize_neighbors,
            process_gas_sharing,
            debug_atmosphere,
        ).chain());
    }
}
