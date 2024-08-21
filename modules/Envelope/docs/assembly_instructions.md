# Envelope Assembly Instructions

See [general assembly instructions](https://quinnfreedman.github.io/modular/docs/assembly)

## Components

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about acquiring parts.

Interactive BOM: [front](https://quinnfreedman.github.io/fm-artifacts/Envelope/rng_pcb_front_interactive_bom.html), [back](https://quinnfreedman.github.io/fm-artifacts/Envelope/rng_pcb_back_interactive_bom.html)

| Missing from Tayda BOM | Board | Reference      | Part             | Value                                   | Source  | Comment |
| ---------------------- | ----- | -------------- | ---------------- | --------------------------------------- | ------- | ------- |
|                        | Front | R1, R3, R5, R7 | Resistor         | 100kΩ                                   | [Tayda](https://www.taydaelectronics.com/10-x-resistor-100k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
|                        | Front | R2, R4, R6, R8 | Resistor         | 200kΩ                                   | [Tayda](https://www.taydaelectronics.com/resistor-200k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
|                        | Front | R9-R12         | Resistor         | 2kΩ                                     | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/test-group-2.html) | Should be 1/5 potentiometer values. If you are using 50kΩ pots, use 10kΩ resistors here. |
|                        | Back  | R13-R20        | Resistor         | 100kΩ                                   |         | |
|                        | Back  | R20-R24        | Resistor         | 10kΩ                                    | [Tayda](https://www.taydaelectronics.com/10-x-resistor-10k-ohm-1-4w-1-metal-film-pkg-of-10.html) | Determines LED brightness. Any value between 220Ω-10kΩ might be appropriate depending on which LEDs you have and how bright you want them. Lower resistance values mean more current and brighter LEDs. |
|                        | Front | R25, R26       | Resistor         | 1Ω                                      | [Tayda](https://www.taydaelectronics.com/10-x-resistor-1k-ohm-1-4w-1-metal-film-pkg-of-10.html) | Determines output impedance |
|                        | Front | RV1-RV4        | Potentiometer    | B10kΩ                                   | [Tayda](https://www.taydaelectronics.com/50k-ohm-linear-taper-potentiometer-d-shaft-pcb-9mm.html), [Thonk](https://www.thonk.co.uk/shop/alpha-9mm-pots-dshaft/) | Linear. A larger value is fine, although if the value is too large the response curve might be a little warped. Make sure to match R9-R12 accordingly. |
| 🔴                     | Front | SW1            | Button           | TL1105SP (e.g. TL1105SPF250Q) + 1RBLK   | [DigiKey (switch)](https://www.digikey.com/en/products/detail/e-switch/TL1105SPF250Q/271559), [DigiKey (cap)](https://www.digikey.com/en/products/detail/e-switch/1RBLK/271579) | The caps for these switches need to be purchased separately. The caps I use are `#1RBLK`. The switches are available in different actuation forces and materials, so the last part of the part number might be a little different. Sometimes, the switches and caps will be sold together and the cap number is appended to the end of the part number. If you don't want to use these specific switches, any "6mm tactile switch" with a standard 4.5mm x 6.5mm mounting pattern like [this one](https://www.taydaelectronics.com/tact-switch-6-6mm-13mm-through-hole-spst-no.html) should work here. Those are a bit narrower, though, so you might want to adjust the faceplate accordingly. |
|                        | Front | J1-J8          | 3.5mm Jack       | THONKICONN (a.k.a PJ398SM or PJ301M-12) | [Tayda](https://www.taydaelectronics.com/pj-3001f-3-5-mm-mono-phone-jack.html), [Thonk](https://www.thonk.co.uk/shop/thonkiconn/) | |
|                        | Front | D1-D11         | LED              | 3mm                                     | [Tayda](https://www.taydaelectronics.com/leds/round-leds/3mm-leds.html) | My design uses 4 colors (amber, blue, red, green) to match the 4 knob colors, but obviously use whatever colors you want. Different colors might have slightly different brightnesses so you could tune R20-R24 to make them uniform. |
|                        | Back  | Q1, Q2         | Transistor       | 2N3904                                  | [Tayda](https://www.taydaelectronics.com/2n3904-npn-general-propose-transistor.html), [DigiKey](https://www.digikey.com/en/products/detail/onsemi/2N3904TA/973944) | |
| 🔴                     | Back  | U1             | DAC              | MCP4922-E/P                             | [DigiKey](https://www.digikey.com/en/products/detail/microchip-technology/MCP4922-E-P/716251) | |
|                        | Back  | U2             | Op-amp           | MCP6004                                 | [Tayda](https://www.taydaelectronics.com/mcp6004-single-supply-cmos-ic.html), [DigiKey](https://www.digikey.com/en/products/detail/microchip-technology/mcp6004-i-p/523060) | |
|                        | Back  | C1             | Capacitor        | 10uF                                    | [Tayda](https://www.taydaelectronics.com/10uf-16v-85c-radial-electrolytic-capacitor.html) | Power supply noise filtering capacitor |
|                        | Back  | C2, C3         | Capacitor        | 100nF                                   | [Tayda](https://www.taydaelectronics.com/a-553-0-1uf-50v-ceramic-disc-capacitor-pkg-of-10.html) | Power supply noise filtering capacitor |
|                        | Back  | A1             | Arduino Nano     | v3.0                                    | [Tayda](https://www.taydaelectronics.com/type-c-nano-3-0-controller-compatible-with-arduino-nano.html) | |
|                        | Both  | J10-J12        | Pin headers      | 1x7, 1x8                                | Tayda ([Male](https://www.taydaelectronics.com/40-pin-2-54-mm-single-row-pin-header-strip.html), [Female](https://www.taydaelectronics.com/40-pin-2-54-mm-single-row-female-pin-header.html)), [Amazon](https://www.amazon.com/gp/product/B074HVBTZ4) | Solder the two boards directly together using the male headers or make them detachable using a male/female pair (recommended). |
|                        | Back  | J13            | IDC connector    | 2x8                                     |         | Eurorack power header. Can use two rows of male pin headers or a shrouded connector (recommended). |
|                        | Back  | J14            | Configuration Jumper |                                     | [Tayda](https://www.taydaelectronics.com/16-pin-box-header-connector-2-54mm.html) | See **Configuration** |

## Configuration

The module has 3 pairs of holes on the reverse side of the back panel (J14). By bridging each pair (either by soldering a wire through both of them or by soldering on pins and using removable jumpers) you can effect the behavior of the module.

Jumpers 1 and 2 control the behavior of the "Aux" output

| Pair 1  | Pair 2  | Aux behavior |
| ------- | ------- | ------------ |
| Open    | Open    | **End-of-rise gate**: Will go high (5v) as soon as the attack stage ends, and will stay high until the cycle ends (output goes to 0) or returns to attack (if the envelope gets re-triggered) |
| Bridged | Open    | **End-of-fall gate**: Will go high at end-of-cycle (after release) and will go low again as soon as the next cycle starts (or after the attack stage in ACRC Loop mode) |
| Open    | Bridged | **Non-zero gate**: Will go high for the duration of the cycle (as long as the envelope is outputting a positive value) |
| Bridged | Bridged | **Gate-follower**: Will duplicate the Gate input |

Config jumper 3 is not currently used. If there is any behavior of the envelope that you would like to be able to customize, let me know!
