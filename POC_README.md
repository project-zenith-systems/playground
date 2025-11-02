# Atmospheric Simulation POC

Proof of concept implementation of the tile-based atmospheric simulation system described in [docs/design/atmospheric-simulation-system.md](docs/design/atmospheric-simulation-system.md).

## Features (POC)

- ✅ Integer-based gas mixture representation (u64, no floating point)
- ✅ Fixed-size array for gas types (better cache locality than HashMap)
- ✅ Basic tile grid with neighbor connections
- ✅ Ideal gas law calculations (pressure from moles, temperature, volume)
- ✅ Standard atmosphere generation
- 🚧 Gas sharing between tiles (TODO)
- 🚧 Heat transfer (TODO)
- 🚧 Dirty tile optimization (TODO)

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

When running, press `SPACE` to print atmospheric data for all tiles in the console.

The POC creates a 3x3 grid of tiles:
- Center tile starts with vacuum
- Surrounding tiles have standard Earth-like atmosphere

## Architecture

```
src/
├── main.rs                    # Application entry point
└── atmosphere/
    ├── mod.rs                 # Module declarations
    ├── gas.rs                 # GasMixture and gas types
    ├── components.rs          # Bevy components (TileAtmosphere, etc.)
    ├── systems.rs             # Bevy systems (gas sharing, etc.)
    └── plugin.rs              # AtmospherePlugin
```

## Next Steps

1. Implement gas sharing algorithm with viscosity-based dampening
2. Implement heat transfer with thermal conductivity
3. Add visual representation of tiles
4. Implement dirty tile optimization
5. Add performance benchmarks
6. Scale up to larger grids (100x100, 1000+ tiles)

## Design Document

See [docs/design/atmospheric-simulation-system.md](docs/design/atmospheric-simulation-system.md) for the complete technical design.
