# TPA2016D2

Embedded-hal Driver for the TI TPA2016D2 Stereo Class-D amplifier.

## Notes

 - Max i2c clock 400 kHz
 - [Datasheet](http://www.ti.com/lit/ds/symlink/tpa2016d2.pdf)
 
 ## Example
```rust
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;

use nucleo_f401re::{i2c::I2c, prelude::*, stm32};
use tpa2016d2::Tpa2016d2;

#[entry]
fn main() -> ! {
    // The Stm32 peripherals
    let device = stm32::Peripherals::take().unwrap();

    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let gpiob = device.GPIOB.split();
    let scl = gpiob
        .pb8
        .into_alternate_af4()
        .internal_pull_up(true)
        .set_open_drain();

    let sda = gpiob
        .pb9
        .into_alternate_af4()
        .internal_pull_up(true)
        .set_open_drain();

    let i2c = I2c::i2c1(device.I2C1, (scl, sda), 200.khz(), clocks);

    let mut tpa = Tpa2016d2::new(i2c);

    // Read all registers
    tpa.sync().unwrap();

    // Get and print  the registers
    for i in 1..=7 {
        let v = tpa.device_reg(i).unwrap();
        hprintln!("{}: {}", i, v).unwrap();
    }

    // Update the gain
    tpa.gain(32).unwrap();

    // Should print 32
    hprintln!("gain: {}", tpa.device_reg(5).unwrap()).unwrap();

    loop {}
}
```

## TODO
 - Agc Presets
 - Features
 