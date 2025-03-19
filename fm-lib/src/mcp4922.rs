use core::convert::Infallible;

use arduino_hal::{prelude::*, Spi};
use embedded_hal::digital::v2::OutputPin;

pub struct MCP4922<PIN>
where
    PIN: OutputPin<Error = Infallible>,
{
    cs_pin: PIN,
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
    /** (default) The channel output is VREF * D/(2^12) where D is the digital value. There is no extra amplification. */
    #[default]
    Unity = 1,
    /** There is an additional 2x amplification to the channel output. The output is 2 * VREF * D/(2^12) where D is the digital value. */
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
    PIN: OutputPin<Error = Infallible>,
{
    pub fn new(cs_pin: PIN) -> Self {
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

    In unbuffered mode (default, BUF=0), VREF input impedance is 165k and input range is
    0V to VDD. In buffered mode (BUF=1), VREF input impedance is higher but range is lower.

    GA is the output gain stage control. When GA=1, output gain is 1x. When GA=0,
    output gain is 2x (relative to VREF).

    SHDN Shut down the given DAC channel when SHDN=0 (output impedance is 500k).

    */
    pub fn write_with_config(
        &mut self,
        spi: &mut Spi,
        channel: DacChannel,
        value: u16,
        config: &ChannelConfig,
    ) {
        self.write_keep_cs_pin_low(spi, channel, value, config);
        self.cs_pin.set_high().unwrap_infallible();
    }

    /**
    _Almost_ writes the data to the DAC, but stops just before releasing the chip
    select. If LDAC is held low (double-buffering disabled) then the value will be
    output by the dac as soon as the CS pin is pulled back high. This function lets
    you do that at a later, precisely timed point, e.g. from an interrupt. Otherwise,
    don't use this function.
    */
    pub fn write_keep_cs_pin_low(
        &mut self,
        spi: &mut Spi,
        channel: DacChannel,
        value: u16,
        config: &ChannelConfig,
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

        self.cs_pin.set_low().unwrap_infallible();
        spi.transfer(&mut [first_byte, second_byte])
            .unwrap_infallible();
    }

    /**
    Write a 12-bit value to the given dac channel while blocking.
    */
    #[inline(always)]
    pub fn write(&mut self, spi: &mut Spi, channel: DacChannel, value: u16) {
        self.write_with_config(spi, channel, value, &ChannelConfig::default())
    }

    pub fn write_both_channels(&mut self, spi: &mut Spi, channel_a: u16, channel_b: u16) {
        let config = Default::default();
        self.write_keep_cs_pin_low(spi, DacChannel::ChannelA, channel_a, &config);
        self.write_keep_cs_pin_low(spi, DacChannel::ChannelB, channel_b, &config);
        self.end_write();
    }

    /**
    Sets the CS pin back high. Only needs to be called if you are using write_keep_cs_pin_low.
    */
    #[inline(always)]
    pub fn end_write(&mut self) {
        self.cs_pin.set_high().unwrap_infallible();
    }

    pub fn shutdown_channel(&mut self, spi: &mut Spi, channel: DacChannel) {
        self.write_with_config(
            spi,
            channel,
            0,
            &ChannelConfig {
                power: Power::Shutdown,
                ..Default::default()
            },
        )
    }
}
