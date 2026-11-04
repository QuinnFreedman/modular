# Clock Assembly Instructions

See [general assembly instructions](https://quinnfreedman.github.io/modular/docs/assembly)

## Components

**Most** parts are available on Tayda ([cart link](https://www.taydaelectronics.com/savecartpro/index/savenewquote/qid/82040858402), [quick-order CSV](https://freemodular.org/modules/Clock/fm_clock_tayda_bom.csv)).

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about acquiring parts.

Interactive BOM: [front](https://quinnfreedman.github.io/fm-artifacts/Clock/clock_front_pcb_interactive_bom.html), [back](https://quinnfreedman.github.io/fm-artifacts/Clock/clock_back_pcb_interactive_bom.html)

|    | Board | Reference | Part             | Value                                   | Source  | Comment |
| -- | ----- | --------- | ---------------- | --------------------------------------- | ------- | ------- |
|    | Front | R1-R8     | Resistor         | 1kΩ                                     | [Tayda](https://www.taydaelectronics.com/10-x-resistor-1k-ohm-1-4w-1-metal-film-pkg-of-10.html) | Determines output impedance. Any value is fine. |
|    | Front | R9-R16    | Resistor         | 5kΩ-10kΩ                                | [Tayda](https://www.taydaelectronics.com/10-x-resistor-10k-ohm-1-4w-1-metal-film-pkg-of-10.html) | Determines LED brightness. You may want to use a different value if you have different LEDs. A lower value means less resistance and brighter LEDs |
|    | Front | D1-D8     | LED              | 3mm                                     | [Tayda](https://www.taydaelectronics.com/36705-dup-led-3mm-blue-diffuse-lens.html) | Any standard 3mm LED will work here. |
|    | Front | SW1       | Rotary Encoder   | EC11 series                             | [Tayda](https://www.taydaelectronics.com/rotary-encoder-11mm-20-detents-d-shaft-with-switch-vertical.html), [Amazon](https://www.amazon.com/dp/B07D3DF8TK) | Some encoders with the same footprint invert the direction of rotation. If after assembly you notice that the encoder is working backwards, hold down the encoder button while powering up the module to invert it. |
| 🔴 | Front | SW2       | Push button      | D6R30 F1 LFS                            | [DigiKey](https://www.digikey.com/en/products/detail/c-k/D6R30-F1-LFS/1466331) | Make sure to line up the flat side of the button with the silkscreen on the PCB. This button could be replaced by a simple "6mm tactile switch" like [this](https://www.taydaelectronics.com/tact-switch-6-6mm-13mm-through-hole-spst-no.html) with a [cap](https://www.taydaelectronics.com/electromechanical/switches-key-pad/tact-switch/34189-dup-tactile-switch-caps-yellow-color.html) on it, but you may need to adjust the faceplate hole size. |
| 🔴 | Front | Screen    | OLED display     | SSD1306 SPI                             | [ebay](https://www.ebay.com/itm/373647815247) | The underlying display controller is the SSD1306. There are many ebay/amazon sellers that make modules based on it. Make sure to get one that uses the SPI protocol (6 or 7 total pins) instead of the I2C protocol (4 pins). The pins may be labelled differently or be in a different order on different modules. Just connect them to the corresponding holes on the PCB. The module also may or may not contain a 7th RESET pin. Either way should be fine. Different modules may also vary in the layout of their mounting holes. The faceplate is designed for a 23.5mmx24mm bolt pattern, which seems to be common, but it could be easily modified to fit a different screen. |
|    | Front | J1-J8     | 3.5mm Jack       | THONKICONN (a.k.a PJ398SM or PJ301M-12) | [Tayda](https://www.taydaelectronics.com/pj-3001f-3-5-mm-mono-phone-jack.html), [Thonk](https://www.thonk.co.uk/shop/thonkiconn/) | |
|    | Front | J9-J11    | Pin headers      | 1x4, 1x4, 1x11                          | Tayda ([Male](https://www.taydaelectronics.com/40-pin-2-54-mm-single-row-pin-header-strip.html), [Female](https://www.taydaelectronics.com/40-pin-2-54-mm-single-row-female-pin-header.html)), [Amazon](https://www.amazon.com/gp/product/B074HVBTZ4) | Solder the two boards directly together using the male headers or make them detachable using a male/female pair. |
|    | Back  | C1        | Capacitor        | -                                       |         | **Optional** additional power supply noise filtering capacitor |
|    | Back  | C2        | Capacitor        | 10uF                                    | [Tayda](https://www.taydaelectronics.com/10uf-16v-85c-radial-electrolytic-capacitor.html) | Power supply noise filtering capacitor |
|    | Back  | A1        | Arduino Nano     | v3.0                                    | [Tayda](https://www.taydaelectronics.com/type-c-nano-3-0-controller-compatible-with-arduino-nano.html), [ebay](https://www.ebay.com/itm/226078083873)| |
|    | Back  | J9-J11    | Pin headers      | 1x4, 1x4, 1x11                          |         | Match corresponding headers on front PCB |
|    | Back  | J12       | IDC connector    | 2x8                                     | [Tayda](https://www.taydaelectronics.com/2x8-pin-2-54-mm-double-row-pin-header-strip.html) | Eurorack power header. Can use two rows of male pin headers or a shrouded connector (recommended). |
|    | Back  | J13       | -                | -                                       |         | Not used. Expansion points for future features |
| 🔴 | Front | -         | Mounting bolts   | 2 M2x8mm, 2 M2x16mm, 6 nuts             | [McMaster-Carr](https://www.mcmaster.com/products/screws/socket-head-screws~/system-of-measurement~metric/thread-size~m2/) | Used to attatch the screen to the faceplate. The two longer bolts can pass through to the PCB using additional nuts to hold everything together more firmly. |

🔴 = Missing from Tayda BOM

## Build Guide

![](images/tutorial_01.jpg)

Once you have gathered all the components and the PCBs, I recommend starting with the resistors.

![](images/tutorial_02.jpg)

Bend the resistor legs back to hold them in place while you solder, then snip off the extra wire.

![](images/tutorial_03.jpg)

Repeat for all resistors. 

![](images/tutorial_04.jpg)

Cut the pin headers to length. Cut between the pins on the male headers but cut over the top of the pins on the female headers.

Stack the boards together and ballance them so that everything sits perpendicular. Solder the joints, then take the boards back apart.

If you prefer, you can solder the boards together permanently using only the male headers. If you do, save this to the very last step so you can still access the solder joints between the boards.

![](images/tutorial_05.jpg)

Repeat the same process for attaching the Arduino to the rear board.

![](images/tutorial_06.jpg)

Attach the power header and capacitor to the rear board. Make sure to align the notch in the power header with the silkscreen. Looking from the back of the module, the notch should be on the right when the module is upright.

![](images/tutorial_07.jpg)

Attach the wires to the screen. If you want, you can use a DuPont connector or a JST XH-2.54 connector to make the screen detachable. Or just solder the wires straight to the screen.

Make sure to trim the tips extra flush so they don't short out against the faceplate when the screen is attached.

![](images/tutorial_08.jpg)

**Be careful connecting the screen to the PCB**. The pins on the screen may not be in the same order or labelled with the same names as they are on the PCB. This display uses the [SPI](https://en.wikipedia.org/wiki/Serial_Peripheral_Interface) protocol, which has very inconsistent naming conventions and every order of the display sub-module I get has a slightly different pinout or labels. There should be 6 (or 7) pins on the screen. Connect each one to the matching connection on the module PCB according to the following table:

| PCB marking | Other abbreviations | Function                |
| ----------- | ------------------- | ----------------------- |
| GND         | 0V, Vss             | ground                  |
| +5V         | 5V, PWR, Vcc        | power                   |
| SCK         | CLK, SCLK, CK, SLK  | serial clock            |
| SDA         | MOSI, PICO, DATA    | serial data             |
| RES         | RESET               | reset (optional*)       |
| DC          | D/C, MODE, DATA     | data/command toggle     |
| CS          | SS, /SS, /CS        | chip select (optional*) |

\* The reset and/or CS pins may not exist on the screen sub-module. If not, just leave them unconnected. If they are present then you need to connect them.

![](images/tutorial_09.jpg)

Attach the screen to the faceplate with M2 bolts. Don't tighten them enough to bend the PCB, just get them gently snug all around.

The connections should be at the top of the module.

![](images/tutorial_10.jpg)

Place all the UI components in the PCB. Make sure to align the flat edge of the button with the silkscreen. The short leg of the LEDs should go in the square holes (toward the bottom of the module).

Add the faceplate before soldering to make sure everything is aligned properly. Screw the jack sockets down tightly. The rotary encoder will not have a surface to tighten the nut against, so don't screw it on tightly or it will pull the boards out of alignment.

Then flip the whole board over and solder. The button might fall out of place since it isn't attached. Make sure to hold it flush while you solder.

![](images/tutorial_11.jpg)

Push the LEDs through so they stick out the desired amount, then solder them too.

![](images/tutorial_12.jpg)

Put the two PCBs back together. Add the knob and the Arduino.

Then flash the bootloader and firmware onto the Arduino, if needed. See the [assembly instructions](https://freemodular.org/docs/assembly) for more details.
