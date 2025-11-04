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
    let mut updates: Vec<(Entity, GasMixture, Vec<(Entity, GasMixture, bool)>, bool)> = Vec::new();
    
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
                }
                // If not there, try active_tiles
                else if let Ok((_, mut neighbor_atmos)) = active_tiles.get_mut(neighbor_entity) {
                    neighbor_atmos.mixture = neighbor_mixture;
                }
                
                // ALWAYS mark neighbor as active when there's a pressure difference
                // This ensures gas continues to spread outward
                commands.entity(neighbor_entity).insert(AtmosphereActive);
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
    tile_query: Query<(Entity, &TilePosition, Option<&Wall>)>,
) {
    // Build a position-to-entity map
    let mut position_map = std::collections::HashMap::new();
    let mut wall_positions = std::collections::HashSet::new();
    
    for (entity, pos, wall) in tile_query.iter() {
        position_map.insert(*pos, entity);
        if wall.is_some() {
            wall_positions.insert(*pos);
        }
    }
    
    // Set up neighbor connections for newly added tiles
    for (pos, mut atmosphere) in query.iter_mut() {
        let neighbor_positions = pos.neighbors();
        
        for (i, neighbor_pos) in neighbor_positions.iter().enumerate() {
            if let Some(&neighbor_entity) = position_map.get(neighbor_pos) {
                // Neighbor is open (not sealed) if neither this tile nor the neighbor is a wall
                let is_open = !wall_positions.contains(pos) && !wall_positions.contains(neighbor_pos);
                atmosphere.neighbors[i] = Some((neighbor_entity, is_open));
            }
        }
    }
}

/// System to update neighbor connections when walls change
pub fn update_wall_connections(
    changed_walls_added: Query<&TilePosition, Added<Wall>>,
    mut changed_walls_removed: RemovedComponents<Wall>,
    mut all_tiles: Query<(&TilePosition, &mut TileAtmosphere, Option<&Wall>)>,
    tile_lookup: Query<(Entity, &TilePosition, Option<&Wall>)>,
) {
    // Check if any walls were added or removed
    let has_added = !changed_walls_added.is_empty();
    let has_removed = changed_walls_removed.read().next().is_some();
    
    if !has_added && !has_removed {
        return;
    }
    
    // Build a position-to-entity map
    let mut position_map = std::collections::HashMap::new();
    let mut wall_positions = std::collections::HashSet::new();
    
    for (entity, pos, wall) in tile_lookup.iter() {
        position_map.insert(*pos, entity);
        if wall.is_some() {
            wall_positions.insert(*pos);
        }
    }
    
    // Update all tiles that might be affected
    for (pos, mut atmosphere, _wall) in all_tiles.iter_mut() {
        let neighbor_positions = pos.neighbors();
        
        for (i, neighbor_pos) in neighbor_positions.iter().enumerate() {
            if let Some(&neighbor_entity) = position_map.get(neighbor_pos) {
                // Neighbor is open if neither this tile nor the neighbor is a wall
                let is_open = !wall_positions.contains(pos) && !wall_positions.contains(neighbor_pos);
                atmosphere.neighbors[i] = Some((neighbor_entity, is_open));
            }
        }
    }
}

/// System to update tile visual representation based on atmospheric pressure
pub fn update_tile_visuals(
    mut query: Query<(&TileAtmosphere, &mut Sprite, Option<&Wall>)>,
) {
    for (atmosphere, mut sprite, wall) in query.iter_mut() {
        // If it's a wall, color it gray
        if wall.is_some() {
            sprite.color = Color::srgb(0.4, 0.4, 0.4);
            continue;
        }
        
        let pressure = atmosphere.mixture.pressure() as f32 / 1_000_000.0; // Convert to kPa
        let standard_pressure = 101.325;
        
        // Color based on pressure relative to standard atmosphere
        // Using logarithmic scale for better visibility of low pressures
        let pressure_ratio = pressure / standard_pressure;
        
        if pressure_ratio < 0.001 {
            // Deep vacuum - black
            sprite.color = Color::srgb(0.0, 0.0, 0.0);
        } else if pressure_ratio < 0.1 {
            // Very low pressure - dark blue to blue (logarithmic scale)
            // Map 0.001-0.1 to 0.2-0.6 intensity
            let log_ratio = (pressure_ratio / 0.001).log10() / 2.0; // 0.0 to 1.0
            let intensity = 0.2 + log_ratio * 0.4;
            sprite.color = Color::srgb(0.0, 0.0, intensity);
        } else if pressure_ratio < 0.5 {
            // Low pressure - blue to bright blue
            let t = (pressure_ratio - 0.1) / 0.4;
            sprite.color = Color::srgb(0.0, t * 0.3, 0.6 + t * 0.4);
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

/// System to calculate flow vectors based on pressure gradients
pub fn calculate_flow_vectors(
    mut tiles: Query<(Entity, &TileAtmosphere, &mut FlowVector, Option<&Wall>)>,
    all_tiles: Query<&TileAtmosphere>,
) {
    // Collect pressure gradients first
    let mut updates = Vec::new();
    
    for (entity, atmosphere, _flow, wall) in tiles.iter() {
        // Skip walls
        if wall.is_some() {
            updates.push((entity, Vec2::ZERO, 0.0));
            continue;
        }
        
        let my_pressure = atmosphere.mixture.pressure() as f32;
        let mut total_gradient = Vec2::ZERO;
        let mut neighbor_count = 0;
        
        // Check all 4 neighbors
        for (i, neighbor_opt) in atmosphere.neighbors.iter().enumerate() {
            if let Some((neighbor_entity, is_open)) = neighbor_opt {
                if *is_open {
                    if let Ok(neighbor_atmos) = all_tiles.get(*neighbor_entity) {
                        let neighbor_pressure = neighbor_atmos.mixture.pressure() as f32;
                        let pressure_diff = neighbor_pressure - my_pressure;
                        
                        // Direction vectors: [North, East, South, West]
                        let direction = match i {
                            0 => Vec2::new(0.0, 1.0),   // North
                            1 => Vec2::new(1.0, 0.0),   // East
                            2 => Vec2::new(0.0, -1.0),  // South
                            3 => Vec2::new(-1.0, 0.0),  // West
                            _ => Vec2::ZERO,
                        };
                        
                        // Gradient points from low to high pressure
                        total_gradient += direction * pressure_diff;
                        neighbor_count += 1;
                    }
                }
            }
        }
        
        if neighbor_count > 0 {
            let magnitude = total_gradient.length();
            let direction = if magnitude > 0.0 {
                total_gradient.normalize()
            } else {
                Vec2::ZERO
            };
            updates.push((entity, direction, magnitude));
        } else {
            updates.push((entity, Vec2::ZERO, 0.0));
        }
    }
    
    // Apply updates
    for (entity, direction, magnitude) in updates {
        if let Ok((_, _, mut flow, _)) = tiles.get_mut(entity) {
            flow.direction = direction;
            flow.magnitude = magnitude;
        }
    }
}
