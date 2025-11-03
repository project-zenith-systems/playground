# Atmospheric Simulation POC

Proof of concept implementation of the tile-based atmospheric simulation system described in [docs/design/atmospheric-simulation-system.md](docs/design/atmospheric-simulation-system.md).

## Features (POC)

- ✅ Integer-based gas mixture representation (u64, no floating point)
- ✅ Fixed-size array for gas types (better cache locality than HashMap)
- ✅ Basic tile grid with neighbor connections
- ✅ Ideal gas law calculations (pressure from moles, temperature, volume)
- ✅ Standard atmosphere generation
- ✅ Gas sharing between tiles with pressure equalization
- ✅ Heat transfer between tiles
- ✅ Visual representation with pressure-based coloring
- ✅ Dirty tile optimization (only updates tiles that changed)

## Building and Running

```bash
# Build the project
cargo build

# Run the POC
cargo run

# Run tests
cargo test
```

## Usage

When running:
- The simulation automatically processes gas sharing between tiles
- Tiles are color-coded based on atmospheric pressure
- Press `SPACE` to print atmospheric data for all tiles in the console

The POC creates a 5x5 grid of tiles:
- Center tile starts with vacuum (black)
- Surrounding tiles have standard Earth-like atmosphere (green)
- Gas gradually flows from high-pressure to low-pressure tiles
- Colors update in real-time to show pressure changes

### Color Legend

- **Black**: Vacuum (< 1% standard pressure)
- **Blue**: Low pressure (1-50% standard)
- **Cyan**: Slightly low (50-90% standard)
- **Green**: Normal pressure (90-110% standard)
- **Yellow/Orange**: High pressure (110-200% standard)
- **Red**: Very high pressure (> 200% standard)

## Architecture

```
src/
├── main.rs                    # Application entry point with visual setup
└── atmosphere/
    ├── mod.rs                 # Module declarations
    ├── gas.rs                 # GasMixture with gas sharing & heat transfer
    ├── components.rs          # Bevy components (TileAtmosphere, etc.)
    ├── systems.rs             # Bevy systems (gas sharing, visualization, etc.)
    └── plugin.rs              # AtmospherePlugin
```

## Implementation Details

### Gas Sharing Algorithm

The system implements a simplified Monson method:
1. Calculate pressure differential between connected tiles
2. Transfer gas proportionally based on pressure difference
3. Transfer heat based on temperature difference
4. Mark affected neighbors as dirty for next update

### Performance

- Uses dirty flagging to only update tiles that changed
- Clones gas mixtures to avoid Rust borrow checker conflicts
- Integer arithmetic throughout (no floating point)
- Fixed-size arrays for better cache performance

## Next Steps

1. ✅ ~~Implement gas sharing algorithm~~
2. ✅ ~~Implement heat transfer~~
3. ✅ ~~Add visual representation of tiles~~
4. Add performance benchmarks
5. Scale up to larger grids (100x100, 1000+ tiles)
6. Implement combustion reactions
7. Add more gas types with different properties
8. Optimize with better algorithms (zone-based updates, etc.)

## Design Document

See [docs/design/atmospheric-simulation-system.md](docs/design/atmospheric-simulation-system.md) for the complete technical design.
