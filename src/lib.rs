//! This is documentation for the `tpa2016d2` module.

#![no_std]
#![allow(dead_code)]

use embedded_hal::blocking::i2c;

mod regmap;
use regmap::*;

// The datasheet uses the adresses 0xB0 and 0xB1 for its examples
// So it is defined like this for clarity.
const TPA2016_I2C_ADDR: u8 = 0xB0 >> 1;

/// Representation of a Texas Instruments TPA2016d2 audio amplifier
pub struct Tpa2016d2<I2C> {
    i2c: I2C,
    regmap: RegisterMap,
}

/// Faults
pub struct Faults {
    pub fault_r: bool,
    pub fault_l: bool,
    pub thermal: bool,
}

/// Compression Ratio
pub enum CompressionRatio {
    /// Ratio 1:1
    Ratio1 = 0b00,
    /// Ratio 2:1
    Ratio2 = 0b01,
    /// Ratio 4:1
    Ratio4 = 0b10,
    /// Ratio 8:1
    Ratio8 = 0b11,
}

/// Noise Gate Threshold
pub enum NoiseGateThreshold {
    Ngt20mV = 0b11,
    Ngt10mV = 0b10,
    Ngt4mV = 0b01,
    Ngt1mV = 0b00,
}

/// Automatic Gain Control Presets
pub enum AgcPreset {
    Pop,
    Classical,
    Jazz,
    Rap,
    Rock,
    Voice,
}

impl<I2C, E> Tpa2016d2<I2C>
where
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
{
    /// Creates a new device connected through the supplied i2c device
    pub fn new(i2c: I2C) -> Tpa2016d2<I2C> {
        let regmap = RegisterMap::default();

        Tpa2016d2 { i2c, regmap }
    }

    /// Read all registers and update our view of the registers
    pub fn sync(&mut self) -> Result<(), E> {
        for i in 1..=7 {
            let val = self.read_reg(i)?;
            self.regmap.update_map(i, val);
        }
        Ok(())
    }

    /// Consume the device and release the i2c device
    pub fn release(self) -> I2C {
        self.i2c
    }

    // Get content of register i
    pub fn device_reg(&mut self, idx: u8) -> Result<u8, E> {
        Ok(self.regmap.reg_as_byte(idx))
    }

    /// Enable or disable speakers
    pub fn speaker_enable(&mut self, le: bool, re: bool) -> Result<(), E> {
        self.regmap.reg1.SPK_EN_L = le;
        self.regmap.reg1.SPK_EN_R = re;
        self.write_regmap_reg(1)
    }

    pub fn get_faults(&mut self) -> Result<Faults, E> {
        // Reload register
        let val = self.read_reg(1)?;
        self.regmap.update_map(1, val);

        Ok(Faults {
            fault_r: self.regmap.reg1.FAULT_R,
            fault_l: self.regmap.reg1.FAULT_L,
            thermal: self.regmap.reg1.Thermal,
        })
    }

    /// Shutdown the device
    /// Control, Bias and Oscillators are disabled
    pub fn disable_device(&mut self) -> Result<(), E> {
        self.regmap.reg1.SWS = true;
        self.write_regmap_reg(1)
    }

    pub fn noise_gate(&mut self, enable: bool) -> Result<(), E> {
        self.regmap.reg1.NG_EN = enable;
        self.write_regmap_reg(1)
    }

    pub fn set_attack_time(&mut self, val: u8) -> Result<(), E> {
        self.regmap.atk_time.set(val);
        self.write_regmap_reg(2)
    }

    /// Set release time / per 6 dB
    pub fn set_release_time(&mut self, val: u8) -> Result<(), E> {
        self.regmap.rel_time.set(val);
        self.write_regmap_reg(3)
    }

    pub fn set_hold_time(&mut self, val: u8) -> Result<(), E> {
        self.regmap.hold_time.set(val);
        self.write_regmap_reg(4)
    }

    /// Set the gain
    pub fn gain(&mut self, gain: u8) -> Result<(), E> {
        self.regmap.fixedGain.set(gain);
        self.write_regmap_reg(5)
    }

    pub fn noise_gate_threshold(&mut self, val: NoiseGateThreshold) -> Result<(), E> {
        self.regmap.reg6.noise_gate_threshold = val as u8;
        self.write_regmap_reg(6)
    }

    pub fn output_limiter_level(&mut self, val: u8) -> Result<(), E> {
        self.regmap.reg6.output_limiter_level = val;
        self.write_regmap_reg(6)
    }

    pub fn compression_ratio(&mut self, ratio: CompressionRatio) -> Result<(), E> {
        self.regmap.reg7.compression_ratio = ratio as u8;
        self.write_regmap_reg(7)
    }

    pub fn set_agc_preset(&mut self, preset: AgcPreset) -> Result<(), E> {
        use AgcPreset::*;
        use CompressionRatio::*;

        // From the data sheet
        let (cr, atk, rel_time, hold_time, fixed_gain, limiter_level) = match preset {
            Pop => (Ratio4, 0b00_0010, 986, 137, 6, 0b11_1100),
            Classical => (Ratio2, 0b00_0010, 1150, 137, 6, 0b11_1101),
            Jazz => (Ratio2, 0b00_0110, 3288, 0, 6, 0b11_1101),
            Rap => (Ratio4, 0b00_0010, 1640, 0, 6, 0b11_1100),
            Rock => (Ratio2, 0b00_0011, 4110, 0, 6, 0b11_1101),
            Voice => (Ratio4, 0b00_0010, 1640, 0, 6, 0b11_1110),
        };

        let rel_time = release_time_to_u6(rel_time);
        let hold_time = hold_time_to_u6(hold_time);

        self.regmap.atk_time.set(atk);
        self.regmap.rel_time.set(rel_time);
        self.regmap.hold_time.set(hold_time);
        self.regmap.fixedGain.set(fixed_gain);
        self.regmap.reg6.output_limiter_level = limiter_level;
        self.regmap.reg7.compression_ratio = cr as u8;

        // Send the new settings to the device
        for rid in 2..=7 {
            self.write_regmap_reg(rid)?;
        }

        Ok(())
    }

    fn write_regmap_reg(&mut self, idx: u8) -> Result<(), E> {
        let b = self.regmap.reg_as_byte(idx);
        self.write_reg(idx, b)
    }

    fn read_reg(&mut self, regidx: u8) -> Result<u8, E> {
        if regidx < 1 || regidx > 7 {
            return Ok(0);
        }

        let mut regbuf = [0u8; 1];
        self.i2c
            .write_read(TPA2016_I2C_ADDR, &[regidx], &mut regbuf)?;

        Ok(regbuf[0])
    }

    fn write_reg(&mut self, regaddr: u8, value: u8) -> Result<(), E> {
        let regbuf = [regaddr, value];
        self.i2c.write(TPA2016_I2C_ADDR, &regbuf)
    }
}

const fn release_time_to_u6(v: u32) -> u8 {
    (v / 1644) as u8
}

const fn hold_time_to_u6(v: u32) -> u8 {
    (v / 137) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_time_conv() {
        let tests = [(1644, 0b00_0001), (4932, 0b00_0011), (103600, 0b11_1111)];
        for &(input, bitval) in &tests {
            let res = release_time_to_u6(input);
            assert_eq!(res, bitval);
        }
    }

    #[test]
    fn hold_time_conv() {
        let tests = [(137, 0b00_0001), (0411, 0b00_0011), (8631, 0b11_1111)];
        for &(input, bitval) in &tests {
            let res = hold_time_to_u6(input);
            assert_eq!(res, bitval);
        }
    }

    #[test]
    fn test_register_defaults() {
        let regmap = RegisterMap::default();

        let r1 = regmap.reg_as_byte(1);
        let r2 = regmap.reg_as_byte(2);
        let r3 = regmap.reg_as_byte(3);
        let r4 = regmap.reg_as_byte(4);
        let r5 = regmap.reg_as_byte(5);
        let r6 = regmap.reg_as_byte(6);
        let r7 = regmap.reg_as_byte(7);

        assert_eq!(r1, 0xC3);
        assert_eq!(r2, 0x05);
        assert_eq!(r3, 0x0B);
        assert_eq!(r4, 0x00);
        assert_eq!(r5, 0x06);
        assert_eq!(r6, 0x3A);
        assert_eq!(r7, 0xC2);
    }
}
