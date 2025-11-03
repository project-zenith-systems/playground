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
    
    /// Share gas with another mixture based on pressure differential
    /// This implements a simplified Monson method for gas equalization
    pub fn share_gas_with(&mut self, other: &mut GasMixture) {
        let pressure_a = self.pressure() as i128;
        let pressure_b = other.pressure() as i128;
        let pressure_diff = pressure_a - pressure_b;
        
        // Only share if significant pressure difference (0.1 kPa = 100,000 μkPa)
        if pressure_diff.abs() < 100_000 {
            return;
        }
        
        let total_moles_a = self.total_moles();
        if total_moles_a == 0 {
            return;
        }
        
        // Calculate transfer amount based on pressure differential
        // Simplified: transfer 10% of the pressure difference worth of gas
        let transfer_moles = (pressure_diff * self.volume as i128) / 
                            (8314 * self.temperature.max(1) as i128 / 100);
        
        // Clamp to prevent numerical instabilities
        let max_transfer = (total_moles_a as i128 / 10).max(1);
        let transfer_moles = transfer_moles.clamp(-max_transfer, max_transfer) as i64;
        
        if transfer_moles == 0 {
            return;
        }
        
        // Transfer each gas proportionally
        for i in 0..GAS_TYPE_COUNT {
            if self.moles[i] == 0 {
                continue;
            }
            
            let ratio = (self.moles[i] as i128 * 1_000_000) / total_moles_a as i128;
            let transfer = (transfer_moles as i128 * ratio) / 1_000_000;
            
            self.moles[i] = (self.moles[i] as i128 - transfer).max(0) as u64;
            other.moles[i] = (other.moles[i] as i128 + transfer).max(0) as u64;
        }
        
        // Also share heat
        self.share_heat_with(other);
    }
    
    /// Share heat with another mixture based on temperature differential
    pub fn share_heat_with(&mut self, other: &mut GasMixture) {
        let total_moles_a = self.total_moles();
        let total_moles_b = other.total_moles();
        
        if total_moles_a == 0 || total_moles_b == 0 {
            return;
        }
        
        // Calculate temperature difference (in milli-Kelvin)
        let temp_diff = self.temperature as i128 - other.temperature as i128;
        
        if temp_diff.abs() < 100 {  // Less than 0.1K difference
            return;
        }
        
        // Simplified heat transfer - transfer proportional to temperature difference
        // In reality this would use thermal conductivity, but for POC we use simplified approach
        let heat_transfer = temp_diff / 10;
        
        self.temperature = (self.temperature as i128 - heat_transfer).max(1) as u64;
        other.temperature = (other.temperature as i128 + heat_transfer).max(1) as u64;
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
    
    #[test]
    fn test_gas_sharing() {
        // Create two mixtures with different pressures
        let mut high_pressure = GasMixture::new_air(STANDARD_VOLUME_MICRO_M3, STANDARD_TEMP_MK);
        let mut low_pressure = GasMixture::new(STANDARD_VOLUME_MICRO_M3, STANDARD_TEMP_MK);
        
        let initial_pressure_high = high_pressure.pressure();
        let initial_pressure_low = low_pressure.pressure();
        
        // Gas should flow from high to low pressure
        assert!(initial_pressure_high > initial_pressure_low);
        
        // Share gas multiple times to simulate equilibration
        for _ in 0..10 {
            high_pressure.share_gas_with(&mut low_pressure);
        }
        
        let final_pressure_high = high_pressure.pressure();
        let final_pressure_low = low_pressure.pressure();
        
        // Pressures should be closer after sharing
        let initial_diff = (initial_pressure_high as i128 - initial_pressure_low as i128).abs();
        let final_diff = (final_pressure_high as i128 - final_pressure_low as i128).abs();
        
        assert!(final_diff < initial_diff, 
            "Pressure difference should decrease after gas sharing. Initial: {}, Final: {}",
            initial_diff, final_diff);
    }
    
    #[test]
    fn test_heat_sharing() {
        let mut hot = GasMixture::new_air(STANDARD_VOLUME_MICRO_M3, 400_000); // ~127°C
        let mut cold = GasMixture::new_air(STANDARD_VOLUME_MICRO_M3, 250_000); // ~-23°C
        
        let initial_temp_hot = hot.temperature;
        let initial_temp_cold = cold.temperature;
        
        // Share heat multiple times
        for _ in 0..10 {
            hot.share_heat_with(&mut cold);
        }
        
        // Temperatures should be closer
        let initial_diff = (initial_temp_hot as i128 - initial_temp_cold as i128).abs();
        let final_diff = (hot.temperature as i128 - cold.temperature as i128).abs();
        
        assert!(final_diff < initial_diff,
            "Temperature difference should decrease after heat sharing");
    }
}
