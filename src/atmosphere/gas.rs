use bevy::prelude::*;

/// Gas type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum GasType {
    Oxygen = 0,
    Nitrogen = 1,
    CarbonDioxide = 2,
    Plasma = 3,
    NitrousOxide = 4,
    WaterVapor = 5,
    Tritium = 6,
}

pub const GAS_TYPE_COUNT: usize = 7;

/// Gas mixture using fixed-size array and integer math for performance
#[derive(Debug, Clone, Component)]
pub struct GasMixture {
    /// Moles of each gas type (stored as micro-moles, 10^-6 mol)
    pub moles: [u64; GAS_TYPE_COUNT],
    
    /// Temperature in milli-Kelvin (mK)
    pub temperature: u64,
    
    /// Volume in micro-cubic meters (μm³)
    pub volume: u64,
}

impl Default for GasMixture {
    fn default() -> Self {
        Self {
            moles: [0; GAS_TYPE_COUNT],
            temperature: 293_150, // 20°C in milli-Kelvin
            volume: 2_500_000, // 2.5 m³ in micro-m³
        }
    }
}

impl GasMixture {
    /// Create a new gas mixture
    pub fn new(volume: u64, temperature: u64) -> Self {
        Self {
            moles: [0; GAS_TYPE_COUNT],
            temperature,
            volume,
        }
    }
    
    /// Create Earth-like atmosphere
    pub fn new_air(volume: u64, temperature: u64) -> Self {
        let mut mixture = Self::new(volume, temperature);
        
        // Calculate total moles for standard pressure (101.325 kPa)
        // P = (n * R * T) / V
        // n = (P * V) / (R * T)
        // P = 101.325 kPa = 101,325,000 μkPa
        // R = 8314 (scaled gas constant)
        // V in μm³, T in mK
        
        let pressure_micro_kpa: u128 = 101_325_000;
        let r_scaled: u128 = 8314;
        
        let total_micromoles = (pressure_micro_kpa * volume as u128 * 1000) / 
                              (r_scaled * temperature as u128);
        
        // 78% N₂, 21% O₂, 1% other (mostly CO₂)
        mixture.moles[GasType::Nitrogen as usize] = ((total_micromoles * 78) / 100) as u64;
        mixture.moles[GasType::Oxygen as usize] = ((total_micromoles * 21) / 100) as u64;
        mixture.moles[GasType::CarbonDioxide as usize] = ((total_micromoles * 1) / 100) as u64;
        
        mixture
    }
    
    /// Calculate total moles
    pub fn total_moles(&self) -> u64 {
        self.moles.iter().sum()
    }
    
    /// Calculate pressure in micro-kPa
    pub fn pressure(&self) -> u64 {
        if self.volume == 0 {
            return 0;
        }
        
        let n = self.total_moles() as u128;
        let r: u128 = 8314;
        let t = self.temperature as u128;
        let v = self.volume as u128;
        
        // P = (n * R * T) / (1000 * V)
        // Division by 1000 accounts for R scaling
        ((n * r * t) / (1000 * v)) as u64
    }
    
    /// Get moles of a specific gas
    pub fn get_moles(&self, gas_type: GasType) -> u64 {
        self.moles[gas_type as usize]
    }
    
    /// Add moles of a specific gas
    pub fn add_moles(&mut self, gas_type: GasType, amount: u64) {
        self.moles[gas_type as usize] = self.moles[gas_type as usize].saturating_add(amount);
    }
    
    /// Remove moles of a specific gas
    pub fn remove_moles(&mut self, gas_type: GasType, amount: u64) {
        self.moles[gas_type as usize] = self.moles[gas_type as usize].saturating_sub(amount);
    }
}

/// Helper constants for unit conversion
pub const MICROMOLES_PER_MOLE: u64 = 1_000_000;
pub const MILLIKELVIN_PER_KELVIN: u64 = 1_000;
pub const MICRO_M3_PER_M3: u64 = 1_000_000;

/// Standard atmospheric constants
pub const STANDARD_PRESSURE_MICRO_KPA: u64 = 101_325_000; // 101.325 kPa
pub const STANDARD_TEMP_MK: u64 = 293_150; // 20°C
pub const STANDARD_VOLUME_MICRO_M3: u64 = 2_500_000; // 2.5 m³ per tile

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gas_mixture_pressure() {
        let mixture = GasMixture::new_air(STANDARD_VOLUME_MICRO_M3, STANDARD_TEMP_MK);
        let pressure = mixture.pressure();
        
        // Should be approximately standard pressure (101.325 kPa = 101,325,000 μkPa)
        // Allow 5% tolerance due to integer math
        let expected = STANDARD_PRESSURE_MICRO_KPA;
        let tolerance = expected / 20; // 5%
        
        assert!(
            pressure > expected - tolerance && pressure < expected + tolerance,
            "Pressure {} should be within {}±{}", 
            pressure, expected, tolerance
        );
    }
    
    #[test]
    fn test_total_moles() {
        let mut mixture = GasMixture::default();
        mixture.add_moles(GasType::Oxygen, 1_000_000);
        mixture.add_moles(GasType::Nitrogen, 2_000_000);
        
        assert_eq!(mixture.total_moles(), 3_000_000);
    }
}
