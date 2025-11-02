use bevy::prelude::*;
use super::components::*;

/// System to process gas sharing between connected tiles
pub fn process_gas_sharing(
    mut query: Query<(Entity, &mut TileAtmosphere), With<AtmosphereDirty>>,
    mut commands: Commands,
) {
    // TODO: Implement gas sharing algorithm
    // For now, just remove the dirty marker
    for (entity, _atmosphere) in query.iter() {
        commands.entity(entity).remove::<AtmosphereDirty>();
    }
}

/// System to initialize neighbor connections
pub fn initialize_neighbors(
    mut query: Query<(&TilePosition, &mut TileAtmosphere), Added<TileAtmosphere>>,
    tile_query: Query<(Entity, &TilePosition)>,
) {
    // Build a position-to-entity map
    let mut position_map = std::collections::HashMap::new();
    for (entity, pos) in tile_query.iter() {
        position_map.insert(*pos, entity);
    }
    
    // Set up neighbor connections for newly added tiles
    for (pos, mut atmosphere) in query.iter_mut() {
        let neighbor_positions = pos.neighbors();
        
        for (i, neighbor_pos) in neighbor_positions.iter().enumerate() {
            if let Some(&neighbor_entity) = position_map.get(neighbor_pos) {
                atmosphere.neighbors[i] = Some((neighbor_entity, true)); // Open by default
            }
        }
    }
}

/// Debug system to print atmospheric data
pub fn debug_atmosphere(
    query: Query<(&TilePosition, &TileAtmosphere)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for (pos, atmosphere) in query.iter() {
            let pressure = atmosphere.mixture.pressure();
            let temp_celsius = (atmosphere.mixture.temperature as f32 / 1000.0) - 273.15;
            println!(
                "Tile ({}, {}): Pressure = {} kPa, Temp = {:.1}°C, Total moles = {} μmol",
                pos.x, pos.y,
                pressure as f32 / 1_000_000.0,
                temp_celsius,
                atmosphere.mixture.total_moles()
            );
        }
    }
}
