use bevy::prelude::*;
use super::components::*;
use super::gas::GasMixture;

/// System to process gas sharing between connected tiles
/// Only processes tiles with AtmosphereActive marker
pub fn process_gas_sharing(
    mut active_tiles: Query<(Entity, &mut TileAtmosphere), With<AtmosphereActive>>,
    mut other_tiles: Query<&mut TileAtmosphere, Without<AtmosphereActive>>,
    mut commands: Commands,
) {
    // Collect updates from active tiles
    let mut updates: Vec<(Entity, GasMixture, Vec<(Entity, GasMixture)>, bool)> = Vec::new();
    
    for (entity, atmosphere) in active_tiles.iter() {
        let mut neighbor_data = Vec::new();
        let mut has_active_exchange = false;
        let my_pressure = atmosphere.mixture.pressure();
        
        for neighbor_opt in atmosphere.neighbors.iter() {
            if let Some((neighbor_entity, is_open)) = neighbor_opt {
                if *is_open {
                    // Try to get from other_tiles (without active marker)
                    if let Ok(neighbor_atmos) = other_tiles.get(*neighbor_entity) {
                        let neighbor_pressure = neighbor_atmos.mixture.pressure();
                        let pressure_diff = (my_pressure as i128 - neighbor_pressure as i128).abs();
                        
                        // Check if there's significant pressure difference (> 0.1 kPa = 100,000 μkPa)
                        if pressure_diff > 100_000 {
                            has_active_exchange = true;
                            neighbor_data.push((*neighbor_entity, neighbor_atmos.mixture.clone(), true));
                        } else {
                            neighbor_data.push((*neighbor_entity, neighbor_atmos.mixture.clone(), false));
                        }
                    }
                    // If not found there, try active_tiles
                    else if let Ok((_, neighbor_atmos)) = active_tiles.get(*neighbor_entity) {
                        let neighbor_pressure = neighbor_atmos.mixture.pressure();
                        let pressure_diff = (my_pressure as i128 - neighbor_pressure as i128).abs();
                        
                        if pressure_diff > 100_000 {
                            has_active_exchange = true;
                            neighbor_data.push((*neighbor_entity, neighbor_atmos.mixture.clone(), true));
                        } else {
                            neighbor_data.push((*neighbor_entity, neighbor_atmos.mixture.clone(), false));
                        }
                    }
                }
            }
        }
        
        if !neighbor_data.is_empty() {
            updates.push((entity, atmosphere.mixture.clone(), neighbor_data, has_active_exchange));
        } else {
            // No neighbors, can't be active
            updates.push((entity, atmosphere.mixture.clone(), vec![], false));
        }
    }
    
    // Process gas sharing for each active tile
    for (tile_entity, mut tile_mixture, neighbor_data, has_active_exchange) in updates {
        for (neighbor_entity, mut neighbor_mixture, had_pressure_diff) in neighbor_data {
            if had_pressure_diff {
                // Share gas between the two mixtures
                tile_mixture.share_gas_with(&mut neighbor_mixture);
                
                // Update the neighbor's mixture
                // First try other_tiles
                if let Ok(mut neighbor_atmos) = other_tiles.get_mut(neighbor_entity) {
                    neighbor_atmos.mixture = neighbor_mixture;
                    // Mark neighbor as active since it just received gas
                    commands.entity(neighbor_entity).insert(AtmosphereActive);
                }
                // If not there, try active_tiles
                else if let Ok((_, mut neighbor_atmos)) = active_tiles.get_mut(neighbor_entity) {
                    neighbor_atmos.mixture = neighbor_mixture;
                }
            }
        }
        
        // Update the tile's mixture
        if let Ok((_, mut tile_atmos)) = active_tiles.get_mut(tile_entity) {
            tile_atmos.mixture = tile_mixture;
        }
        
        // Remove active marker if no active exchange with any neighbor
        if !has_active_exchange {
            commands.entity(tile_entity).remove::<AtmosphereActive>();
        }
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

/// Component to mark tiles for visual rendering
#[derive(Component)]
pub struct TileVisual;

/// System to update tile visual representation based on atmospheric pressure
pub fn update_tile_visuals(
    mut query: Query<(&TileAtmosphere, &mut Sprite), With<TileVisual>>,
) {
    for (atmosphere, mut sprite) in query.iter_mut() {
        let pressure = atmosphere.mixture.pressure() as f32 / 1_000_000.0; // Convert to kPa
        let standard_pressure = 101.325;
        
        // Color based on pressure relative to standard atmosphere
        let pressure_ratio = pressure / standard_pressure;
        
        if pressure_ratio < 0.01 {
            // Vacuum - black
            sprite.color = Color::srgb(0.0, 0.0, 0.0);
        } else if pressure_ratio < 0.5 {
            // Low pressure - blue
            let intensity = pressure_ratio * 2.0;
            sprite.color = Color::srgb(0.0, 0.0, intensity);
        } else if pressure_ratio < 0.9 {
            // Slightly low - cyan
            let intensity = (pressure_ratio - 0.5) / 0.4;
            sprite.color = Color::srgb(0.0, intensity, 1.0);
        } else if pressure_ratio < 1.1 {
            // Normal - green
            sprite.color = Color::srgb(0.0, 1.0, 0.0);
        } else if pressure_ratio < 2.0 {
            // High pressure - yellow to orange
            let intensity = (pressure_ratio - 1.1) / 0.9;
            sprite.color = Color::srgb(1.0, 1.0 - intensity * 0.5, 0.0);
        } else {
            // Very high pressure - red
            sprite.color = Color::srgb(1.0, 0.0, 0.0);
        }
    }
}
