mod atmosphere;

use bevy::prelude::*;
use atmosphere::{AtmospherePlugin, components::*};

const TILE_SIZE: f32 = 32.0;
const GRID_SIZE: i32 = 25;

/// Marker component for flow arrow visuals
#[derive(Component)]
struct FlowArrow {
    parent_tile: Entity,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AtmospherePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_tile_click, visualize_flow_arrows))
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
            
            let tile_entity = commands.spawn((
                atmosphere,
                TilePosition::new(x, y),
                FlowVector::default(),
                Sprite {
                    color: Color::srgb(0.5, 0.5, 0.5),
                    custom_size: Some(Vec2::new(TILE_SIZE - 1.0, TILE_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0),
            )).id();
            
            // Add wall component if this is a wall
            if has_wall {
                commands.entity(tile_entity).insert(Wall);
            } else if is_center {
                // Mark center as active to start gas flow
                commands.entity(tile_entity).insert(AtmosphereActive);
            }
            
            // Spawn flow arrow as a child entity
            commands.spawn((
                FlowArrow { parent_tile: tile_entity },
                Sprite {
                    color: Color::srgba(1.0, 1.0, 0.0, 0.6),
                    custom_size: Some(Vec2::new(2.0, 16.0)),
                    ..default()
                },
                Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 1.0),
                Visibility::Hidden,
            ));
        }
    }
    
    println!("Atmospheric simulation initialized!");
    println!("Created 25x25 grid with air in center, surrounded by walls");
    println!("Click on tiles to toggle walls");
    println!("\nColor legend:");
    println!("  Black: Deep vacuum");
    println!("  Dark Blue: Very low pressure (logarithmic scale)");
    println!("  Blue: Low pressure");
    println!("  Cyan: Slightly low");
    println!("  Green: Normal pressure");
    println!("  Yellow/Orange: High pressure");
    println!("  Red: Very high pressure");
    println!("  Gray: Wall");
    println!("\nYellow arrows show gas flow direction and strength");
}

/// System to visualize flow vectors with arrows
fn visualize_flow_arrows(
    tiles: Query<(Entity, &FlowVector, &Transform, Option<&Wall>)>,
    mut arrows: Query<(&FlowArrow, &mut Transform, &mut Visibility, &mut Sprite), Without<FlowVector>>,
) {
    for (arrow, mut arrow_transform, mut visibility, mut sprite) in arrows.iter_mut() {
        if let Ok((_, flow, tile_transform, wall)) = tiles.get(arrow.parent_tile) {
            // Don't show arrows for walls or zero-magnitude flows
            if wall.is_some() || flow.magnitude < 100_000.0 {
                *visibility = Visibility::Hidden;
                continue;
            }
            
            // Show arrow
            *visibility = Visibility::Visible;
            
            // Position at tile center
            arrow_transform.translation = tile_transform.translation;
            arrow_transform.translation.z = 1.0; // Draw on top
            
            // Rotate arrow to point in flow direction
            if flow.direction.length() > 0.0 {
                let angle = flow.direction.y.atan2(flow.direction.x) - std::f32::consts::PI / 2.0;
                arrow_transform.rotation = Quat::from_rotation_z(angle);
            }
            
            // Scale arrow based on magnitude (logarithmic for better visibility)
            let normalized_magnitude = (flow.magnitude / 1_000_000.0).min(10.0); // Cap at 10x standard pressure
            let length = 8.0 + normalized_magnitude.log10().max(0.0) * 8.0;
            sprite.custom_size = Some(Vec2::new(2.0, length));
            
            // Color based on magnitude
            let intensity = (normalized_magnitude / 10.0).min(1.0);
            sprite.color = Color::srgba(1.0, 1.0 - intensity * 0.5, 0.0, 0.7);
        }
    }
}

/// System to handle mouse clicks on tiles to toggle walls
fn handle_tile_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    tiles: Query<(Entity, &Transform, &TilePosition, Option<&Wall>)>,
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
        for (entity, transform, pos, wall) in tiles.iter() {
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
