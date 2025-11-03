mod atmosphere;

use bevy::prelude::*;
use atmosphere::{AtmospherePlugin, components::*, systems::TileVisual};

const TILE_SIZE: f32 = 64.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AtmospherePlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2d);
    
    // Create a simple 5x5 grid of tiles for testing
    for x in -2..=2 {
        for y in -2..=2 {
            let atmosphere = if x == 0 && y == 0 {
                // Center tile starts with vacuum
                TileAtmosphere::new_vacuum()
            } else {
                // Other tiles have air
                TileAtmosphere::new_with_air()
            };
            
            commands.spawn((
                atmosphere,
                TilePosition::new(x, y),
                AtmosphereActive, // Mark as active for initial processing
                TileVisual,
                Sprite {
                    color: Color::srgb(0.5, 0.5, 0.5),
                    custom_size: Some(Vec2::new(TILE_SIZE - 2.0, TILE_SIZE - 2.0)),
                    ..default()
                },
                Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0),
            ));
        }
    }
    
    println!("Atmospheric simulation initialized!");
    println!("Press SPACE to print atmospheric data for all tiles");
    println!("Created 5x5 grid with vacuum in center and air in surrounding tiles");
    println!("\nColor legend:");
    println!("  Black: Vacuum");
    println!("  Blue: Low pressure");
    println!("  Cyan: Slightly low");
    println!("  Green: Normal pressure");
    println!("  Yellow/Orange: High pressure");
    println!("  Red: Very high pressure");
}
