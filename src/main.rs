mod atmosphere;

use bevy::prelude::*;
use atmosphere::{AtmospherePlugin, components::*, systems::TileVisual};

const TILE_SIZE: f32 = 32.0;
const GRID_SIZE: i32 = 25;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AtmospherePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_tile_click)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2d);
    
    let half_size = GRID_SIZE / 2;
    
    // Create a 25x25 grid
    for x in -half_size..=half_size {
        for y in -half_size..=half_size {
            let is_center = x == 0 && y == 0;
            let is_wall_ring = (x.abs() == 1 || y.abs() == 1) && x.abs() <= 1 && y.abs() <= 1;
            
            let (atmosphere, has_wall) = if is_center {
                // Center tile has air
                (TileAtmosphere::new_with_air(), false)
            } else if is_wall_ring {
                // Ring around center is walls (with vacuum)
                (TileAtmosphere::new_vacuum(), true)
            } else {
                // Everything else is vacuum
                (TileAtmosphere::new_vacuum(), false)
            };
            
            let mut entity_commands = commands.spawn((
                atmosphere,
                TilePosition::new(x, y),
                TileVisual,
                Sprite {
                    color: Color::srgb(0.5, 0.5, 0.5),
                    custom_size: Some(Vec2::new(TILE_SIZE - 1.0, TILE_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0),
            ));
            
            // Add wall component if this is a wall
            if has_wall {
                entity_commands.insert(Wall);
            } else if is_center {
                // Mark center as active to start gas flow
                entity_commands.insert(AtmosphereActive);
            }
        }
    }
    
    println!("Atmospheric simulation initialized!");
    println!("Created 25x25 grid with air in center, surrounded by walls");
    println!("Click on tiles to toggle walls");
    println!("\nColor legend:");
    println!("  Black: Vacuum");
    println!("  Blue: Low pressure");
    println!("  Cyan: Slightly low");
    println!("  Green: Normal pressure");
    println!("  Yellow/Orange: High pressure");
    println!("  Red: Very high pressure");
    println!("  Gray: Wall");
}

/// System to handle mouse clicks on tiles to toggle walls
fn handle_tile_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut tiles: Query<(Entity, &Transform, &TilePosition, Option<&Wall>, &TileAtmosphere), With<TileVisual>>,
    mut commands: Commands,
    mut tile_atmosphere: Query<&mut TileAtmosphere>,
    all_tiles: Query<(Entity, &TilePosition)>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    
    let window = windows.single();
    let (camera, camera_transform) = cameras.single();
    
    if let Some(cursor_pos) = window.cursor_position() {
        // Convert cursor position to world coordinates
        let window_size = Vec2::new(window.width(), window.height());
        
        // Convert screen position to world position
        // Note: Bevy's cursor position has Y=0 at top, but world has Y=0 at center
        let mut ndc = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
        ndc.y = -ndc.y; // Flip Y axis to match world coordinates
        
        let ndc_to_world = camera_transform.compute_matrix() * camera.clip_from_view().inverse();
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        let world_pos = world_pos.truncate();
        
        // Find which tile was clicked
        for (entity, transform, pos, wall, _atmosphere) in tiles.iter() {
            let tile_pos = Vec2::new(transform.translation.x, transform.translation.y);
            let half_size = TILE_SIZE / 2.0;
            
            if world_pos.x >= tile_pos.x - half_size && world_pos.x <= tile_pos.x + half_size &&
               world_pos.y >= tile_pos.y - half_size && world_pos.y <= tile_pos.y + half_size {
                
                // Don't allow toggling the center tile
                if pos.x == 0 && pos.y == 0 {
                    continue;
                }
                
                // Toggle wall
                if wall.is_some() {
                    // Remove wall
                    commands.entity(entity).remove::<Wall>();
                    println!("Removed wall at ({}, {})", pos.x, pos.y);
                    
                    // Mark tile and all neighbors as active to trigger gas flow
                    commands.entity(entity).insert(AtmosphereActive);
                    
                    // Activate all neighboring tiles
                    let neighbor_positions = pos.neighbors();
                    for neighbor_pos in neighbor_positions.iter() {
                        for (neighbor_entity, neighbor_tile_pos) in all_tiles.iter() {
                            if neighbor_tile_pos == neighbor_pos {
                                commands.entity(neighbor_entity).insert(AtmosphereActive);
                            }
                        }
                    }
                } else {
                    // Add wall
                    commands.entity(entity).insert(Wall);
                    
                    // Clear atmosphere when wall is added
                    if let Ok(mut atmos) = tile_atmosphere.get_mut(entity) {
                        atmos.mixture = atmosphere::gas::GasMixture::new(
                            atmosphere::gas::STANDARD_VOLUME_MICRO_M3,
                            2_700
                        );
                    }
                    println!("Added wall at ({}, {})", pos.x, pos.y);
                    
                    // Mark tile as active to trigger recalculation
                    commands.entity(entity).insert(AtmosphereActive);
                }
                
                break;
            }
        }
    }
}

                
                break;
            }
        }
    }
}
