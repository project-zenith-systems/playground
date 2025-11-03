use bevy::prelude::*;
use super::gas::GasMixture;

/// Attached to each tile entity
#[derive(Component)]
pub struct TileAtmosphere {
    pub mixture: GasMixture,
    pub sealed: bool,
    /// Neighbor tiles and their connection state (open/closed)
    /// [North, East, South, West]
    pub neighbors: [Option<(Entity, bool)>; 4],
}

impl Default for TileAtmosphere {
    fn default() -> Self {
        Self {
            mixture: GasMixture::default(),
            sealed: true,
            neighbors: [None; 4],
        }
    }
}

impl TileAtmosphere {
    /// Create a new tile with air
    pub fn new_with_air() -> Self {
        use super::gas::{STANDARD_VOLUME_MICRO_M3, STANDARD_TEMP_MK};
        Self {
            mixture: GasMixture::new_air(STANDARD_VOLUME_MICRO_M3, STANDARD_TEMP_MK),
            sealed: true,
            neighbors: [None; 4],
        }
    }
    
    /// Create a new vacuum tile
    pub fn new_vacuum() -> Self {
        use super::gas::STANDARD_VOLUME_MICRO_M3;
        Self {
            mixture: GasMixture::new(STANDARD_VOLUME_MICRO_M3, 2_700), // ~2.7K
            sealed: false,
            neighbors: [None; 4],
        }
    }
}

/// Marker component - presence indicates tile has active gas exchange with neighbors
/// Tile remains active until equilibrium is reached with all neighbors
#[derive(Component)]
pub struct AtmosphereActive;

/// Space/void marker
#[derive(Component)]
pub struct ExposedToSpace;

/// Wall marker - tiles with this component block gas flow
#[derive(Component)]
pub struct Wall;

/// Tile position in the grid
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilePosition {
    pub x: i32,
    pub y: i32,
}

impl TilePosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    /// Get neighboring positions in cardinal directions
    pub fn neighbors(&self) -> [TilePosition; 4] {
        [
            TilePosition::new(self.x, self.y + 1), // North
            TilePosition::new(self.x + 1, self.y), // East
            TilePosition::new(self.x, self.y - 1), // South
            TilePosition::new(self.x - 1, self.y), // West
        ]
    }
}
