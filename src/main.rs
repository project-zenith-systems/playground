mod atmosphere;

use bevy::prelude::*;
use atmosphere::{AtmospherePlugin, components::*, gas::*};

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
    
    // Create a simple 3x3 grid of tiles for testing
    for x in -1..=1 {
        for y in -1..=1 {
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
                AtmosphereDirty, // Mark as dirty for initial processing
            ));
        }
    }
    
    println!("Atmospheric simulation initialized!");
    println!("Press SPACE to print atmospheric data for all tiles");
    println!("Created 3x3 grid with vacuum in center and air in surrounding tiles");
}
