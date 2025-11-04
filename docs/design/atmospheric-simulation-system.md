# Atmospheric Simulation System - Technical Design Document

**Version:** 1.0  
**Date:** November 2025  
**Author:** Project Zenith Systems  
**Status:** Design Phase

## Executive Summary

This document outlines the technical design for a tile-based atmospheric simulation system for a space station game inspired by Space Station 13, compatible with the Bevy game engine. The system aims to simulate air and other gases with reasonable accuracy while maintaining performance suitable for real-time gameplay.

The design prioritizes:
- **Performance**: Efficient simulation suitable for large space stations (1000+ tiles)
- **Accuracy**: Physically plausible gas behavior without full CFD complexity
- **Flexibility**: Support for various gases, temperature, and pressure
- **Integration**: Seamless compatibility with Bevy ECS architecture
- **Gameplay**: Balance between realism and fun, emergent gameplay opportunities

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Architecture Design](#architecture-design)
3. [Technical Specifications](#technical-specifications)
4. [Bevy Integration](#bevy-integration)
5. [Simulation Algorithm](#simulation-algorithm)
6. [Performance Optimization](#performance-optimization)
7. [Data Structures](#data-structures)
8. [API Design](#api-design)
9. [Testing Strategy](#testing-strategy)
10. [Implementation Roadmap](#implementation-roadmap)
11. [Future Enhancements](#future-enhancements)

---

## System Overview

### Scope

The atmospheric simulation system manages the behavior of gases within a tile-based space station environment. It simulates:

- **Gas Mixtures**: Multiple gas types (O₂, N₂, CO₂, plasma, etc.) per tile
- **Pressure**: Equalization and propagation across tiles
- **Temperature**: Heat transfer and thermodynamics
- **Flow**: Gas movement through vents, doors, breaches
- **Reactions**: Combustion, toxicity, and other gas interactions

### Design Philosophy

The system follows a **hybrid approach** balancing:
1. **Cellular Automata**: For efficient local tile updates
2. **Flow Network**: For pressure equalization and long-range effects
3. **Event-Driven**: For catastrophic events (hull breaches, fires)

This hybrid model provides better performance than pure computational fluid dynamics (CFD) while maintaining sufficient accuracy for gameplay.

---

## Architecture Design

### High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│              Bevy Game Engine                       │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌───────────────┐         ┌──────────────┐       │
│  │  Atmosphere   │◄────────┤   Events     │       │
│  │   System      │         │  (Breaches,  │       │
│  │               │         │   Fires)     │       │
│  └───────┬───────┘         └──────────────┘       │
│          │                                         │
│          ├─────► Gas Manager                       │
│          │       - Gas types & properties          │
│          │       - Chemical reactions              │
│          │                                         │
│          ├─────► Tile Grid                         │
│          │       - Spatial partitioning            │
│          │       - Neighbor access                 │
│          │                                         │
│          ├─────► Simulation Scheduler              │
│          │       - Update frequency control        │
│          │       - Priority zones                  │
│          │                                         │
│          └─────► Rendering & Debug                 │
│                  - Gas overlay visualization       │
│                  - Pressure/temp display           │
└─────────────────────────────────────────────────────┘
```

### Component Hierarchy

1. **AtmospherePlugin**: Main Bevy plugin integrating all systems
2. **TileAtmosphere**: Component attached to each tile entity
3. **AtmosphereConfig**: Global simulation parameters
4. **GasReaction**: Chemical reaction definitions
5. **EnvironmentEffects**: Effects on entities (damage, vision, etc.)

---

## Technical Specifications

### Gas Model

Each tile contains a gas mixture defined by:

```rust
/// Fixed-size array approach for better cache locality and performance.
/// Since we have a known set of gas types, we can use a fixed array indexed by GasType.
pub struct GasMixture {
    /// Moles of each gas type (mol) - indexed by GasType as usize
    /// Using fixed array instead of HashMap for better performance with thousands of tiles
    pub moles: [u64; GAS_TYPE_COUNT],  // Stored as micro-moles (10^-6 mol) to avoid floating point
    
    /// Temperature in milli-Kelvin (mK) - stored as u64 to avoid floating point inaccuracies
    pub temperature: u64,
    
    /// Volume in micro-cubic meters (μm³) - stored as u64
    pub volume: u64,
}

// Helper constants
pub const GAS_TYPE_COUNT: usize = 7;  // Number of GasType variants
pub const MICROMOLES_PER_MOLE: u64 = 1_000_000;
pub const MILLIKELVIN_PER_KELVIN: u64 = 1_000;
pub const MICRO_M3_PER_M3: u64 = 1_000_000;
```

#### Gas Types

```rust
pub enum GasType {
    Oxygen,        // O₂ - Breathable, supports combustion
    Nitrogen,      // N₂ - Inert filler gas
    CarbonDioxide, // CO₂ - Byproduct, toxic in high concentration
    Plasma,        // Custom gas - highly reactive, valuable
    NitrousOxide,  // N₂O - Oxidizer, sedative effects
    WaterVapor,    // H₂O - Humidity, fog effects
    Tritium,       // Radioactive, glowing effects
}
```

#### Gas Properties

```rust
pub struct GasProperties {
    /// Molar mass in micro-grams per mole (μg/mol)
    pub molar_mass: u64,
    
    /// Specific heat capacity in micro-Joules per (mol·K) (μJ/(mol·K))
    pub specific_heat: u64,
    
    /// Heat capacity ratio (Cp/Cv) scaled by 1000 for integer storage
    pub heat_capacity_ratio: u64,
    
    /// Fusion temperature in milli-Kelvin (mK)
    pub fusion_temp: Option<u64>,
    
    /// Thermal conductivity (affects heat transfer between tiles)
    /// Scaled by 10^6 for integer storage
    pub thermal_conductivity: u64,
    
    /// Dynamic viscosity (affects gas flow dampening)
    /// Scaled by 10^9 for integer storage (nano-Pascal-seconds)
    pub dynamic_viscosity: u64,
    
    /// Is this gas oxidizer?
    pub is_oxidizer: bool,
    
    /// Is this gas fuel?
    pub is_fuel: bool,
    
    /// Toxicity threshold in micro-kPa partial pressure
    pub toxicity_threshold: Option<u64>,
}
```

### Physical Calculations

#### Pressure (Ideal Gas Law)

Using integer arithmetic with appropriate scaling:

```
P = (n * R * T) / V

Where (in integer units):
- P = Pressure in micro-kPa (μkPa)
- n = Total micro-moles
- R = Gas constant = 8314 (scaled: actual 8.314 J/(mol·K) * 1000)
- T = Temperature in milli-Kelvin (mK)
- V = Volume in micro-cubic meters (μm³)

// Example calculation avoiding overflow:
// P_micro_kPa = (n_micromoles * 8314 * T_millikelvin) / (1000 * V_micro_m3)
```

The math works with u64 by using scaled integer representations. This avoids floating-point inaccuracies while maintaining precision suitable for gameplay.

#### Heat Capacity

```
Heat Capacity = Σ(micromoles_i × specific_heat_i) / MICROMOLES_PER_MOLE
```

#### Gas Sharing (Pressure Equalization)

When two tiles share gas:
1. Calculate total moles and heat capacity
2. Calculate new shared temperature (weighted by heat capacity)
3. Distribute moles proportionally by volume and connectivity
4. Apply flow resistance based on connection type

---

## Bevy Integration

### Plugin Structure

```rust
pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AtmosphereConfig>()
            .init_resource::<GasRegistry>()
            .add_event::<AtmosphereEvent>()
            
            // Simulation systems
            .add_systems(
                FixedUpdate,
                (
                    update_tile_atmospheres,
                    process_gas_flow,
                    handle_heat_transfer,
                    process_reactions,
                    apply_environmental_effects,
                )
                .chain()
                .in_set(AtmosphereSimulationSet)
            )
            
            // Rendering/debug systems
            .add_systems(
                Update,
                (
                    update_atmosphere_visualization,
                    debug_atmosphere_overlay,
                )
            );
    }
}
```

### Core Components

```rust
/// Attached to each tile entity
#[derive(Component)]
pub struct TileAtmosphere {
    pub mixture: GasMixture,
    pub sealed: bool,  // Is this tile sealed (walls, doors)?
    /// Neighbor tiles and their connection state (open/closed)
    /// Using a fixed-size array for up to 4 cardinal directions
    pub neighbors: [Option<(Entity, bool)>; 4],  // N, E, S, W
}

/// Space/void marker
#[derive(Component)]
pub struct ExposedToSpace;
```

### Events

```rust
#[derive(Event)]
pub enum AtmosphereEvent {
    /// Sudden decompression event
    Breach {
        tile: Entity,
        severity: u64,  // Rate of gas loss in micromoles per tick
    },
    
    /// Fire started
    Ignition {
        tile: Entity,
        fuel_micromoles: u64,
    },
}
```

---

## Simulation Algorithm

### Update Cycle (Fixed Timestep)

The simulation runs at a fixed rate (default: 1-2 Hz) separate from the render loop.

```
For each simulation tick:
  1. Process Equipment
     - Vents add/remove gas
     - Scrubbers filter gases
     - Heaters/coolers adjust temperature
  
  2. Process Gas Sharing
     - Calculate pressure differentials
     - Share gas between connected tiles
     - Handle space exposure
  
  3. Process Heat Transfer
     - Conduct heat between tiles
     - Radiate heat to space
     - Apply equipment heating/cooling
  
  4. Process Reactions
     - Check ignition conditions
     - Process combustion
     - Handle special reactions
  
  5. Apply Effects
     - Damage entities in hazardous atmospheres
     - Update visibility (smoke, fog)
     - Trigger events
```

### Gas Sharing Algorithm

**Monson Method** (optimized for games):

```rust
fn share_gas(tile_a: &mut GasMixture, tile_b: &mut GasMixture, is_open: bool, gas_props: &GasRegistry) {
    if !is_open {
        return;
    }
    
    let total_volume = tile_a.volume + tile_b.volume;
    
    // Calculate pressures (in micro-kPa)
    let pressure_a = tile_a.pressure();
    let pressure_b = tile_b.pressure();
    let pressure_diff = pressure_a as i128 - pressure_b as i128;
    
    // Only share if significant pressure difference (0.1 kPa = 100,000 μkPa)
    if pressure_diff.abs() < 100_000 {
        return;
    }
    
    // Calculate transfer amount based on pressure differential
    // Using integer math throughout to avoid floating point
    let transfer_moles = (pressure_diff * tile_a.volume as i128) / 
                         (8314 * tile_a.temperature as i128 / 1000);
    
    // Calculate average viscosity of the gas mixture for dampening
    let viscosity_a = tile_a.average_viscosity(gas_props);
    let viscosity_b = tile_b.average_viscosity(gas_props);
    let avg_viscosity = (viscosity_a + viscosity_b) / 2;
    
    // Dampen transfer based on gas viscosity (higher viscosity = slower flow)
    // Viscosity is in nano-Pascal-seconds, scale appropriately
    let viscosity_factor = 1_000_000_000 / (avg_viscosity + 1_000_000); // Avoid div by zero
    let dampened_transfer = (transfer_moles * viscosity_factor as i128) / 1000;
    
    // Clamp to prevent numerical instabilities
    let max_transfer = (tile_a.total_moles() as i128 / 10).min(tile_b.total_moles() as i128 / 10);
    let transfer_moles = dampened_transfer.clamp(-max_transfer, max_transfer) as i64;
    
    if transfer_moles == 0 {
        return;
    }
    
    // Transfer each gas proportionally
    let total_moles_a = tile_a.total_moles();
    for i in 0..GAS_TYPE_COUNT {
        if tile_a.moles[i] == 0 {
            continue;
        }
        
        let ratio = (tile_a.moles[i] as i128 * 1_000_000) / total_moles_a as i128;
        let transfer = (transfer_moles as i128 * ratio) / 1_000_000;
        
        tile_a.moles[i] = (tile_a.moles[i] as i128 - transfer).max(0) as u64;
        tile_b.moles[i] = (tile_b.moles[i] as i128 + transfer).max(0) as u64;
    }
    
    // Transfer heat
    share_heat(tile_a, tile_b, is_open, gas_props);
}
```

### Heat Transfer

**Thermal Conduction**:

Heat transfer between tiles is based on the thermal conductivity of the gas mixtures, not arbitrary dampening factors.

```rust
fn share_heat(tile_a: &mut GasMixture, tile_b: &mut GasMixture, is_open: bool, gas_props: &GasRegistry) {
    if !is_open {
        return;
    }
    
    let heat_capacity_a = tile_a.heat_capacity(gas_props);
    let heat_capacity_b = tile_b.heat_capacity(gas_props);
    
    if heat_capacity_a == 0 || heat_capacity_b == 0 {
        return;
    }
    
    // Calculate temperature difference (in milli-Kelvin)
    let temp_diff = tile_a.temperature as i128 - tile_b.temperature as i128;
    
    if temp_diff.abs() < 100 {  // Less than 0.1K difference
        return;
    }
    
    // Calculate average thermal conductivity of both mixtures
    let conductivity_a = tile_a.average_thermal_conductivity(gas_props);
    let conductivity_b = tile_b.average_thermal_conductivity(gas_props);
    let avg_conductivity = (conductivity_a + conductivity_b) / 2;
    
    // Heat transfer based on Fourier's law, scaled by thermal conductivity
    // Q = k * A * ΔT / d, simplified for tile-based system
    // Higher conductivity = faster heat transfer
    let heat_transfer = (avg_conductivity as i128 * temp_diff) / 100_000_000;
    
    // Update temperatures based on heat capacities
    let delta_temp_a = heat_transfer / heat_capacity_a as i128;
    let delta_temp_b = heat_transfer / heat_capacity_b as i128;
    
    tile_a.temperature = (tile_a.temperature as i128 - delta_temp_a).max(0) as u64;
    tile_b.temperature = (tile_b.temperature as i128 + delta_temp_b).max(0) as u64;
}
```

### Combustion

```rust
fn process_combustion(mixture: &mut GasMixture) -> Option<FireEvent> {
    let oxygen_moles = mixture.moles.get(&GasType::Oxygen).unwrap_or(&0.0);
    let plasma_moles = mixture.moles.get(&GasType::Plasma).unwrap_or(&0.0);
    
    // Check ignition conditions
    if mixture.temperature < PLASMA_IGNITION_TEMP || 
       *oxygen_moles < MIN_OXYGEN_FOR_FIRE ||
       *plasma_moles < MIN_PLASMA_FOR_FIRE {
        return None;
    }
    
    // Calculate reaction rate
    let reaction_efficiency = (mixture.temperature / PLASMA_IGNITION_TEMP - 1.0)
        .clamp(0.0, 1.0);
    
    let moles_burned = plasma_moles.min(*oxygen_moles * 2.0) * 
                       reaction_efficiency * FIRE_RATE;
    
    // Consume reactants
    *mixture.moles.get_mut(&GasType::Plasma).unwrap() -= moles_burned;
    *mixture.moles.get_mut(&GasType::Oxygen).unwrap() -= moles_burned * 0.5;
    
    // Produce products and heat
    *mixture.moles.entry(GasType::CarbonDioxide).or_insert(0.0) += 
        moles_burned * 0.75;
    
    let heat_released = moles_burned * PLASMA_BURN_ENERGY;
    mixture.temperature += heat_released / mixture.heat_capacity();
    
    Some(FireEvent {
        intensity: moles_burned,
        heat_released,
    })
}
```

---

## Performance Optimization

### Spatial Partitioning

**Zone-Based Updates**:

Divide the station into zones and update zones based on activity:

```rust
pub enum ZonePriority {
    Critical,  // Update every tick (fires, breaches)
    Active,    // Update every 2-3 ticks (occupied areas)
    Stable,    // Update every 5-10 ticks (sealed, equilibrium)
    Inactive,  // Update every 20+ ticks (empty, stable)
}
```

### Active Tile Optimization

Only update tiles that have active gas flow. The presence of the component indicates the tile has non-equalized gas exchange with neighbors.

```rust
/// Marker component - presence indicates tile has active gas exchange
#[derive(Component)]
pub struct AtmosphereActive;
```

A tile is marked as Active when:
- It's initially created with gas
- A neighbor's pressure differs significantly (> 0.1 kPa)
- Gas sharing occurs with any neighbor

A tile's Active marker is removed when:
- All neighbors are equalized (pressure differences < 0.1 kPa)
- The tile has no gas and all neighbors have no gas

This is more efficient than scanning all tiles every frame - only tiles with active gas exchange are processed.

```rust
// Mark tile as active when gas sharing occurs
commands.entity(tile_entity).insert(AtmosphereActive);

// Process only active tiles
fn process_gas_sharing(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TileAtmosphere), With<AtmosphereActive>>,
    all_tiles: Query<&TileAtmosphere>,
) {
    for (entity, mut atmosphere) in query.iter_mut() {
        let mut has_active_exchange = false;
        
        // Process gas sharing with neighbors
        // ...
        
        // Check if still active after processing
        for neighbor in &atmosphere.neighbors {
            if has_pressure_difference(atmosphere, neighbor) {
                has_active_exchange = true;
                break;
            }
        }
        
        // Remove active marker if equilibrium reached
        if !has_active_exchange {
            commands.entity(entity).remove::<AtmosphereActive>();
        }
    }
}
```

### Multithreading

Process tiles in parallel, but only active tiles to avoid wasted work:

```rust
fn process_gas_sharing(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TileAtmosphere), With<AtmosphereActive>>,
    all_tiles: Query<&TileAtmosphere>,
) {
    // Process active tiles
    // Each tile's neighbors array allows independent processing
    // Tiles share gas with neighbors, which may cause write conflicts,
    // but Bevy's change detection will mark affected neighbors as active
    // for the next frame, ensuring eventual consistency
    
    query.par_iter_mut().for_each(|(entity, mut atmosphere)| {
        // Process each active tile
        // Read neighbor data and update atmosphere
        for (neighbor_entity, is_open) in atmosphere.neighbors.iter().flatten() {
            if *is_open {
                // Would need neighbor's atmosphere data here
                // In practice, this requires a two-pass approach or
                // careful synchronization to avoid data races
            }
        }
    });
}
```

Note: Actual implementation requires careful handling of neighbor updates to avoid data races. A practical approach is to:
1. First pass: Calculate transfers in parallel (read-only)
2. Second pass: Apply transfers sequentially or with synchronization
3. Mark neighbors as active when changes occur
4. Remove active marker when equilibrium is reached with all neighbors

### Gas Mixture Pooling

Reuse gas mixture allocations to reduce memory allocations during simulation:

```rust
#[derive(Resource)]
pub struct GasMixturePool {
    /// Pool of reusable gas mixture instances
    pool: Vec<GasMixture>,
}

impl GasMixturePool {
    /// Borrow a gas mixture from the pool for temporary calculations
    pub fn acquire(&mut self) -> GasMixture {
        self.pool.pop().unwrap_or_else(GasMixture::default)
    }
    
    /// Return a gas mixture to the pool for reuse
    pub fn release(&mut self, mut mixture: GasMixture) {
        // Reset to default state
        mixture.moles.fill(0);
        mixture.temperature = 0;
        mixture.volume = 0;
        
        // Only keep a reasonable number in the pool
        if self.pool.len() < 1000 {
            self.pool.push(mixture);
        }
    }
}
```

This is particularly useful when calculating intermediate values during gas sharing or combustion calculations, reducing allocator pressure.

### Update Budgeting

Limit updates per frame:

```rust
#[derive(Resource)]
pub struct AtmosphereConfig {
    pub max_tiles_per_frame: usize,  // e.g., 1000
    pub target_updates_per_second: f32,  // e.g., 2.0
}
```

---

## Data Structures

### Tile Grid

**Sparse Grid** for memory efficiency:

```rust
#[derive(Resource)]
pub struct AtmosphereGrid {
    /// Map of position to tile entity
    tiles: HashMap<IVec2, Entity>,
    
    /// Spatial hash for quick neighbor lookup
    spatial_hash: SpatialHash<Entity>,
    
    /// Cached neighbor connections
    neighbor_cache: HashMap<Entity, Vec<(Entity, f32)>>,
}

impl AtmosphereGrid {
    pub fn get_neighbors(&self, pos: IVec2) -> Vec<(Entity, f32)> {
        // Return cached neighbors with flow coefficients
    }
    
    pub fn invalidate_neighbors(&mut self, pos: IVec2) {
        // Clear cache when tile changes (door opens/closes)
    }
}
```

### Gas Registry

**Flyweight Pattern** for gas properties:

```rust
#[derive(Resource)]
pub struct GasRegistry {
    properties: HashMap<GasType, GasProperties>,
    reaction_table: HashMap<(GasType, GasType), Reaction>,
}

impl GasRegistry {
    pub fn get_properties(&self, gas: GasType) -> &GasProperties {
        &self.properties[&gas]
    }
    
    pub fn register_reaction(&mut self, reactants: (GasType, GasType), 
                            reaction: Reaction) {
        self.reaction_table.insert(reactants, reaction);
    }
}
```

---

## API Design

### Public API

```rust
// Initialize the atmosphere system
app.add_plugins(AtmospherePlugin::default());

// Create a tile with atmosphere
commands.spawn((
    TileAtmosphere {
        mixture: GasMixture::new_air(STANDARD_VOLUME, STANDARD_TEMP),
        sealed: true,
    },
    AtmosphereConnection::default(),
));

// Expose a tile to space
commands.entity(tile_entity)
    .insert(ExposedToSpace);

// Add a vent
commands.entity(tile_entity)
    .insert(AtmosphereEquipment::Vent {
        target_pressure: 101.325, // Standard atmosphere
    });

// Query atmospheric conditions
fn check_breathable(
    query: Query<&TileAtmosphere>,
    player_query: Query<&Position, With<Player>>,
) {
    for position in player_query.iter() {
        if let Some(atmosphere) = query.get(position.tile_entity).ok() {
            let oxygen_pp = atmosphere.mixture.partial_pressure(GasType::Oxygen);
            if oxygen_pp < MIN_BREATHABLE_OXYGEN {
                // Player is suffocating!
            }
        }
    }
}
```

### Configuration

```rust
#[derive(Resource)]
pub struct AtmosphereConfig {
    // Simulation parameters
    pub simulation_rate: f32,  // Hz
    pub gas_share_coefficient: f32,
    pub heat_transfer_coefficient: f32,
    
    // Performance settings
    pub max_tiles_per_frame: usize,
    pub enable_multithreading: bool,
    pub zone_update_strategy: ZoneUpdateStrategy,
    
    // Gameplay settings
    pub enable_fire: bool,
    pub enable_toxicity: bool,
    pub realism_level: RealismLevel,
}

pub enum RealismLevel {
    Arcade,    // Fast, simplified
    Balanced,  // Default, good compromise
    Simulation,  // More realistic, slower
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ideal_gas_law() {
        let mut mixture = GasMixture::default();
        mixture.add_gas(GasType::Oxygen, 1.0);
        mixture.temperature = 273.15;
        mixture.volume = 0.0224;
        
        assert_approx_eq!(mixture.pressure(), 101.325, 0.1);
    }
    
    #[test]
    fn test_gas_sharing() {
        let mut tile_a = GasMixture::new_air(1.0, 293.0);
        let mut tile_b = GasMixture::default();
        tile_b.volume = 1.0;
        tile_b.temperature = 293.0;
        
        let initial_pressure = tile_a.pressure();
        
        share_gas(&mut tile_a, &mut tile_b, 1.0);
        
        // Pressure should equalize
        assert!(tile_a.pressure() < initial_pressure);
        assert!(tile_b.pressure() > 0.0);
    }
    
    #[test]
    fn test_combustion() {
        let mut mixture = GasMixture::default();
        mixture.volume = 1.0;
        mixture.temperature = PLASMA_IGNITION_TEMP + 100.0;
        mixture.add_gas(GasType::Plasma, 10.0);
        mixture.add_gas(GasType::Oxygen, 20.0);
        
        let result = process_combustion(&mut mixture);
        
        assert!(result.is_some());
        assert!(mixture.temperature > PLASMA_IGNITION_TEMP);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_pressure_equalization_system() {
    let mut app = App::new();
    app.add_plugins(AtmospherePlugin::default());
    
    // Create two connected tiles
    let tile_a = app.world.spawn((
        TileAtmosphere {
            mixture: GasMixture::new_air(1.0, 293.0),
            sealed: true,
        },
    )).id();
    
    let tile_b = app.world.spawn((
        TileAtmosphere {
            mixture: GasMixture::default(),
            sealed: true,
        },
    )).id();
    
    // Connect them
    app.world.entity_mut(tile_a)
        .insert(AtmosphereConnection {
            connected_tiles: vec![tile_b],
            flow_coefficient: 1.0,
        });
    
    // Run simulation
    for _ in 0..100 {
        app.update();
    }
    
    // Check pressures are equalized
    let atmos_a = app.world.get::<TileAtmosphere>(tile_a).unwrap();
    let atmos_b = app.world.get::<TileAtmosphere>(tile_b).unwrap();
    
    assert_approx_eq!(
        atmos_a.mixture.pressure(),
        atmos_b.mixture.pressure(),
        1.0
    );
}
```

### Performance Benchmarks

```rust
fn benchmark_atmosphere_update(c: &mut Criterion) {
    let mut app = App::new();
    app.add_plugins(AtmospherePlugin::default());
    
    // Create 1000 tiles
    for _ in 0..1000 {
        app.world.spawn(TileAtmosphere {
            mixture: GasMixture::new_air(1.0, 293.0),
            sealed: true,
        });
    }
    
    c.bench_function("atmosphere_update_1000_tiles", |b| {
        b.iter(|| {
            app.update();
        });
    });
}
```

---

## Implementation Roadmap

### Phase 1: Core Foundation (2-3 weeks)

- [x] Set up project structure
- [ ] Implement basic gas mixture data structure
- [ ] Implement ideal gas law calculations
- [ ] Create Bevy plugin skeleton
- [ ] Add tile atmosphere component
- [ ] Implement basic gas sharing algorithm
- [ ] Unit tests for gas physics

**Deliverable**: Basic gas pressure equalization between tiles

### Phase 2: Grid & Connectivity (1-2 weeks)

- [ ] Implement atmosphere grid
- [ ] Add neighbor detection system
- [ ] Create connection component
- [ ] Handle door opening/closing
- [ ] Add space exposure support
- [ ] Integration tests for grid

**Deliverable**: Connected tile network with proper gas flow

### Phase 3: Temperature & Heat (1-2 weeks)

- [ ] Add temperature to gas mixture
- [ ] Implement heat capacity calculations
- [ ] Create heat transfer algorithm
- [ ] Add heater/cooler equipment
- [ ] Temperature-based effects
- [ ] Heat transfer tests

**Deliverable**: Thermal simulation with equipment

### Phase 4: Reactions & Fire (2 weeks)

- [ ] Implement combustion system
- [ ] Add ignition conditions
- [ ] Create fire spread mechanics
- [ ] Add fire suppression (CO₂, depressurization)
- [ ] Visual effects for fire
- [ ] Combustion tests

**Deliverable**: Working fire simulation

### Phase 5: Equipment & Control (1-2 weeks)

- [ ] Implement vents (add/remove gas)
- [ ] Implement scrubbers (filter gases)
- [ ] Add pressure/temperature sensors
- [ ] Create control interface
- [ ] Equipment integration tests

**Deliverable**: Atmospheric control systems

### Phase 6: Optimization (2-3 weeks)

- [ ] Implement zone-based priorities
- [ ] Add dirty flagging
- [ ] Optimize gas sharing algorithm
- [ ] Implement multithreading
- [ ] Add update budgeting
- [ ] Performance benchmarks
- [ ] Profile and optimize hotspots

**Deliverable**: Optimized system for 2000+ tiles

### Phase 7: Polish & Effects (1-2 weeks)

- [ ] Add atmospheric hazard effects
- [ ] Implement suffocation/toxicity damage
- [ ] Create visual overlays (pressure, temperature)
- [ ] Add sound effects (alarms, decompression)
- [ ] Debug visualization tools
- [ ] Player feedback systems

**Deliverable**: Complete gameplay integration

### Phase 8: Documentation & Tools (1 week)

- [ ] API documentation
- [ ] Usage examples
- [ ] Tutorial for game integration
- [ ] Debug console commands
- [ ] Configuration presets

**Deliverable**: Production-ready system

---

## Future Enhancements

### Advanced Features

1. **Gas Phase Changes**: Condensation/evaporation for water vapor
2. **Advanced Chemistry**: More complex reactions (rust, corrosion)
3. **Wind/Flow Dynamics**: Directional gas movement for wind effects
4. **Radiation**: Heat radiation between tiles
5. **Humidity**: Moisture tracking for environmental effects
6. **Gas Leaks**: Slow leaks from damaged pipes/walls
7. **Pressure Damage**: Structural damage from extreme pressure
8. **Atmospheric Composition Analysis**: In-game tools for players

### Optimization Opportunities

1. **GPU Compute**: Offload simulation to GPU for massive parallelism
2. **SIMD**: Vectorize gas calculations
3. **LOD System**: Reduce simulation fidelity for distant areas
4. **Delta Compression**: Network optimization for multiplayer
5. **Predictive Simulation**: Extrapolate stable areas

### Gameplay Extensions

1. **Life Support Metrics**: Overall station health dashboard
2. **Emergency Systems**: Automated fire suppression, bulkhead sealing
3. **Gas Mining**: Extract valuable gases from planets/asteroids
4. **Terraforming**: Long-term atmospheric management
5. **Environmental Suits**: Player equipment interaction

---

## References

### Technical Resources

- **Space Station 13 Atmospheric System**: [SS13 Wiki - Atmospherics](https://tgstation13.org/wiki/Guide_to_Atmospherics)
- **Ideal Gas Law**: [Wikipedia](https://en.wikipedia.org/wiki/Ideal_gas_law)
- **Thermodynamics**: Heat capacity, enthalpy, entropy
- **Bevy ECS**: [Bevy Book](https://bevyengine.org/learn/book/)
- **Game Physics**: Real-Time Collision Detection, Christer Ericson

### Similar Implementations

- **SS13/SS14**: Open-source reference implementation
- **Oxygen Not Included**: Commercial gas simulation
- **Dwarf Fortress**: Complex fluid simulation
- **Barotrauma**: Underwater pressure simulation

---

## Appendix

### A. Gas Constants

```rust
pub const GAS_CONSTANT: f32 = 8.314; // J/(mol·K)
pub const STANDARD_PRESSURE: f32 = 101.325; // kPa
pub const STANDARD_TEMP: f32 = 293.15; // K (20°C)
pub const STANDARD_VOLUME: f32 = 2.5; // m³ per tile
pub const MIN_PRESSURE_DIFF: f32 = 0.1; // kPa
pub const MIN_TEMP_DIFF: f32 = 0.1; // K
```

### B. Combustion Parameters

```rust
pub const PLASMA_IGNITION_TEMP: f32 = 373.15; // K (100°C)
pub const MIN_OXYGEN_FOR_FIRE: f32 = 0.1; // mol
pub const MIN_PLASMA_FOR_FIRE: f32 = 0.1; // mol
pub const PLASMA_BURN_ENERGY: f32 = 5000.0; // J/mol
pub const FIRE_RATE: f32 = 0.1; // fraction per tick
```

### C. Standard Atmospheres

```rust
impl GasMixture {
    /// Earth-like atmosphere at sea level
    pub fn new_air(volume: f32, temperature: f32) -> Self {
        let mut mixture = Self {
            moles: HashMap::new(),
            temperature,
            volume,
        };
        
        // 78% N₂, 21% O₂, 1% other
        let total_moles = (STANDARD_PRESSURE * volume) / 
                         (GAS_CONSTANT * temperature);
        mixture.add_gas(GasType::Nitrogen, total_moles * 0.78);
        mixture.add_gas(GasType::Oxygen, total_moles * 0.21);
        mixture.add_gas(GasType::CarbonDioxide, total_moles * 0.01);
        
        mixture
    }
    
    /// Vacuum/space
    pub fn new_vacuum(volume: f32) -> Self {
        Self {
            moles: HashMap::new(),
            temperature: 2.7, // Cosmic background radiation
            volume,
        }
    }
}
```

### D. Performance Targets

| Metric | Target | Stretch Goal |
|--------|--------|--------------|
| Tiles Simulated | 1000 | 5000 |
| Update Rate | 1-2 Hz | 5 Hz |
| Frame Budget | < 5ms | < 2ms |
| Memory per Tile | < 1KB | < 500B |
| Latency to Equilibrium | < 10s | < 5s |

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | Nov 2025 | Project Zenith Systems | Initial design document |

---

## Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Technical Lead | | | |
| Game Designer | | | |
| Performance Engineer | | | |

---

**End of Document**
