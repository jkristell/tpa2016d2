#![no_std]
#![allow(dead_code)]

use embedded_hal::blocking::i2c;

mod regmap;
use regmap::*;

const TPA2016_I2C_ADDR: u8 = 0xB0 >> 1;

/// Representation of a Texas Instruments TPA2016 Audio Amplifier
pub struct Tpa2016<I2C> {
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
    Ngt4mV  = 0b01,
    Ngt1mV  = 0b00,
}

impl<I2C, E> Tpa2016<I2C>
where 
    I2C: i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
{
    /// Creates a new device connected through the supplied i2c device
    pub fn new(i2c: I2C) -> Tpa2016<I2C> {
        let regmap = RegisterMap::default();

        Tpa2016 {
            i2c,
            regmap,
        }
    }

    /// Consumes the device and releases the i2c device
    pub fn release(self) -> I2C {
        self.i2c
    }

    // Read register
    pub fn read_device_reg(&mut self, idx: u8) -> Result<u8, E> {
        self.read_reg(idx)
    }

    /// Enable or disable speakers
    pub fn speaker_enable(&mut self, le: bool, re: bool)  -> Result<(), E> {
        self.regmap.reg1.SPK_EN_L = le;
        self.regmap.reg1.SPK_EN_R = re;
        let b = self.regmap.reg_as_byte(1);
        self.write_reg(1, b)
    }

    pub fn get_faults(&mut self) -> Result<Faults, E> {
        let v = self.read_reg(1)?;
        self.regmap.reg1.update_from(v);

        Ok(Faults {
            fault_r: self.regmap.reg1.FAULT_R,
            fault_l: self.regmap.reg1.FAULT_L,
            thermal: self.regmap.reg1.Thermal,
        })
    }

    /// Shutdown the device
    /// Control, Bias and Oscillators are disabled
    pub fn disable_device(&mut self)  -> Result<(), E> {
        self.regmap.reg1.SWS = true;
        self.write_reg_idx(1)
    }

    pub fn noise_gate(&mut self, enable: bool)  -> Result<(), E> {
        self.regmap.reg1.NG_EN = enable;
        self.write_reg_idx(1)
    }

    /// Sets the Volume
    pub fn gain(&mut self, gain: u8)  -> Result<(), E> {
        self.regmap.fixedGain.set(gain);
        self.write_reg_idx(5)
    }

    /// Set attack time / Per 6 dB
    pub fn set_attack_time(&mut self, us: u16) -> Result<(), E> {
        let regval = (us / 1280) as u8;
        self.regmap.atk_time.set(regval);
        self.write_reg_idx(2)
    }

    /// Set release time / per 6 dB
    pub fn set_release_time(&mut self, ms: u16) -> Result<(), E> {
        let regval = (ms / 164) as u8;
        self.regmap.rel_time.set(regval);
        self.write_reg_idx(3)
    }

    pub fn set_hold_time(&mut self, ms: u16) -> Result<(), E> {
        let val = (ms / 164) as u8;
        self.regmap.hold_time.set(val);
        self.write_reg_idx(4)
    }

    pub fn set_fixed_gain(&mut self, db: u8) -> Result<(), E> {
        self.regmap.fixedGain.set(db);
        self.write_reg_idx(5)
    }

    pub fn noise_gate_threshold(&mut self, val: NoiseGateThreshold)  -> Result<(), E> {
        self.regmap.reg6.noise_gate_threshold = val as u8;
        self.write_reg_idx(6)
    }

    pub fn output_limiter_level(&mut self, val: u8) -> Result<(), E> {
        self.regmap.reg6.output_limiter_level = val;
        self.write_reg_idx(6)
    }

    pub fn compression_ratio(&mut self, ratio: CompressionRatio) -> Result<(), E> {
        self.regmap.reg7.compression_ratio = ratio as u8;
        self.write_reg_idx(7)
    }

    fn write_reg_idx(&mut self, idx: usize) -> Result<(), E> {
        let b = self.regmap.reg_as_byte(idx);
        self.write_reg(idx as u8, b)
    }

    fn read_reg(&mut self, regidx: u8) -> Result<u8, E> {
        if regidx < 1 || regidx > 7 {
            return Ok(0);
        }

        let mut regbuf = [0u8; 1];
        self.i2c.write_read(TPA2016_I2C_ADDR, &[regidx], &mut regbuf)?;

        return Ok(regbuf[0]);
    }

    fn write_reg(&mut self, regaddr: u8, value: u8) -> Result<(), E> {
        let regbuf = [regaddr, value];
        self.i2c.write(TPA2016_I2C_ADDR, &regbuf)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

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
