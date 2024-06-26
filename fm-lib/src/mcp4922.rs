use arduino_hal::{
    port::{mode::Output, Pin, PinOps},
    prelude::*,
    Spi,
};

pub struct MCP4922<PIN>
where
    PIN: PinOps,
{
    cs_pin: Pin<Output, PIN>,
}

#[derive(Copy, Clone)]
pub enum DacChannel {
    /** Operation should effect channel A */
    ChannelA = 0,
    /** Operation should effect channel B */
    ChannelB = 1,
}

#[derive(Copy, Clone, Default)]
pub enum BufferMode {
    /** (default) In unbuffered mode, input impedance is ~165k and input range is 0V to VDD. */
    #[default]
    Unbuffered = 0,
    /** In buffered mode, input impedance is higher but input range is lower. */
    Buffered = 1,
}

#[derive(Copy, Clone, Default)]
pub enum Power {
    /** (default) The given channel is enabled as an output. Operating current is ~700uA@5v w/ 5kR to GND.  */
    #[default]
    Enabled = 1,
    /** The given channel is disabled. Output impedance is 500k and power draw is minimal. Operating current is ~6uA@5v w/ 5kR to GND. */
    Shutdown = 0,
}

#[derive(Copy, Clone, Default)]
pub enum MultiplierMode {
    /** (default) The channel output is VREF * D/(2^n) where D is the digtial value. There is no extra amplification. */
    #[default]
    Unity = 1,
    /** There is an additional 2x amplification to the channel output. The output is 2 * VREF * D/(2^n) where D is the digtial value. */
    Double = 0,
}

#[derive(Default)]
pub struct ChannelConfig {
    pub buffer_mode: BufferMode,
    pub power: Power,
    pub multiplier_mode: MultiplierMode,
}

impl<PIN> MCP4922<PIN>
where
    PIN: PinOps,
{
    pub fn new(cs_pin: Pin<Output, PIN>) -> Self {
        MCP4922 { cs_pin }
    }

    /**
    Write a 12-bit value to the given dac channel while blocking.

    Interface:

    Bit 15                                      Bit 0
    +-----+-----+-----+------+-----+-----+-----+----+
    | DAC | BUF | GA  | SHDN | D12 | D11 | ... | D1 |
    +-----+-----+-----+------+-----+-----+-----+----+

    DAC selects which channel to write to.

    In unbuffered mode (default, BUF=0), input impedance is 165k and input range is
    0V to VDD. In puffered mode (BUF=1), input impedance is higher but range is lower.

    GA is the output gain stage control. When GA=1, output gain is 1x. When GA=0,
    output gain is 2x (relative to VREF).

    SHDN Shut down the given DAC channel when SHDN=0 (output impedance is 500k).

    */
    pub fn write_sync(
        &mut self,
        spi: &mut Spi,
        channel: DacChannel,
        value: u16,
        config: ChannelConfig,
    ) {
        let data_low: u8 = value as u8;
        let data_high: u8 = (value >> 8) as u8;
        debug_assert!(data_high <= 0xf);

        let dac_bit = (channel as u8) << 7;
        let buf_bit = (config.buffer_mode as u8) << 6;
        let ga_bit = (config.multiplier_mode as u8) << 5;
        let shdn_bit = (config.power as u8) << 4;

        let first_byte = dac_bit | buf_bit | ga_bit | shdn_bit | data_high;
        let second_byte = data_low;

        self.cs_pin.set_low();
        spi.transfer(&mut [first_byte, second_byte])
            .unwrap_infallible();
        self.cs_pin.set_high();
    }

    /**
    Write a 12-bit value to the given dac channel while blocking.
    */
    #[inline(always)]
    pub fn write(&mut self, spi: &mut Spi, channel: DacChannel, value: u16) {
        self.write_sync(spi, channel, value, ChannelConfig::default())
    }
}
