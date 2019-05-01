pub trait RegisterMapRegister {
    fn as_byte(&self) -> u8;
    fn update(&mut self, val: u8);
}

#[allow(non_snake_case)]
pub struct Register1 {
    pub SPK_EN_R: bool,
    pub SPK_EN_L: bool,
    pub SWS: bool,
    pub FAULT_R: bool,
    pub FAULT_L: bool,
    pub Thermal: bool,
    pub NG_EN: bool,
}

impl Default for Register1 {
    fn default() -> Self {
        Self {
            SPK_EN_R: true,
            SPK_EN_L: true,
            SWS: false,
            FAULT_R: false,
            FAULT_L: false,
            Thermal: false,
            NG_EN: true,
        }
    }
}

impl RegisterMapRegister for Register1 {
    fn as_byte(&self) -> u8 {
        let mut r = 0;

        if self.SPK_EN_R {
            r |= 1 << 7;
        }
        if self.SPK_EN_L {
            r |= 1 << 6;
        }
        if self.SWS {
            r |= 1 << 5;
        }
        if self.FAULT_R {
            r |= 1 << 4;
        }
        if self.FAULT_L {
            r |= 1 << 3;
        }
        if self.Thermal {
            r |= 1 << 2;
        }
        // Bit 1 should always be 1
        r |= 1 << 1;
        if self.NG_EN {
            r |= 1;
        }

        r
    }

    fn update(&mut self, new: u8) {
        self.SPK_EN_R = new & 1 << 7 != 0;
        self.SPK_EN_L = new & 1 << 6 != 0;
        self.SWS = new & 1 << 5 != 0;
        self.FAULT_R = new & 1 << 4 != 0;
        self.FAULT_L = new & 1 << 3 != 0;
        self.Thermal = new & 1 << 2 != 0;
        // Reserved
        self.NG_EN = new & 1 != 0;
    }
}

pub struct U6Register(u8);

impl U6Register {
    pub fn set(&mut self, value: u8) {
        self.0 = value & 0x3F;
    }
}

impl RegisterMapRegister for U6Register {
    fn as_byte(&self) -> u8 {
        self.0 & 0x3F
    }

    fn update(&mut self, val: u8) {
        self.0 = val & 0x3F;
    }
}

pub struct Register6 {
    pub output_limiter_disable: bool,
    pub noise_gate_threshold: u8,
    pub output_limiter_level: u8,
}

impl Default for Register6 {
    fn default() -> Self {
        Self {
            output_limiter_disable: false,
            noise_gate_threshold: 0b01,
            output_limiter_level: 0b11010,
        }
    }
}

impl RegisterMapRegister for Register6 {
    fn as_byte(&self) -> u8 {
        let mut r = 0;

        if self.output_limiter_disable {
            r |= 1 << 7
        }

        r |= (self.noise_gate_threshold & 0b11) << 5;
        r |= self.output_limiter_level & 0b11111;
        r
    }

    fn update(&mut self, val: u8) {
        self.output_limiter_disable = val & 1 << 7 != 0;
        self.noise_gate_threshold = (val >> 5) & 0b11;
        self.output_limiter_level = val & 0b11111;
    }
}

pub struct Register7 {
    pub max_gain: u8,
    pub compression_ratio: u8,
}

impl Default for Register7 {
    fn default() -> Self {
        Self {
            max_gain: 0b1100,
            compression_ratio: 0b10,
        }
    }
}

impl RegisterMapRegister for Register7 {
    fn as_byte(&self) -> u8 {
        // Gain
        (self.max_gain & 0b1111) << 4 |
        // Compression radio
        (self.compression_ratio & 0b11)
    }

    fn update(&mut self, val: u8) {
        self.max_gain = val >> 4;
        self.compression_ratio = val & 0b11;
    }
}

#[allow(non_snake_case)]
pub struct RegisterMap {
    pub reg1: Register1,
    pub atk_time: U6Register,  // reg2
    pub rel_time: U6Register,  // reg3
    pub hold_time: U6Register, // reg4
    pub fixedGain: U6Register, // reg5
    pub reg6: Register6,
    pub reg7: Register7,
}

impl Default for RegisterMap {
    fn default() -> Self {
        Self {
            reg1: Register1::default(),
            atk_time: U6Register(0x05),
            rel_time: U6Register(0x0B),
            hold_time: U6Register(0x00),
            fixedGain: U6Register(0x06),
            reg6: Register6::default(),
            reg7: Register7::default(),
        }
    }
}

impl RegisterMap {
    pub fn reg_as_byte(&self, idx: usize) -> u8 {
        match idx {
            1 => self.reg1.as_byte(),
            2 => self.atk_time.as_byte(),
            3 => self.rel_time.as_byte(),
            4 => self.hold_time.as_byte(),
            5 => self.fixedGain.as_byte(),
            6 => self.reg6.as_byte(),
            7 => self.reg7.as_byte(),
            _ => 0,
        }
    }
}
