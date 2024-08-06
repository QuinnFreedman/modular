# Envelope Assembly Instructions

See [general assembly instructions](https://quinnfreedman.github.io/modular/docs/assembly)

## Components

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about acquiring parts.

Interactive BOM: [front](https://quinnfreedman.github.io/fm-artifacts/Envelope/rng_pcb_front_interactive_bom.html), [back](https://quinnfreedman.github.io/fm-artifacts/Envelope/rng_pcb_back_interactive_bom.html)

| Board | Reference      | Part             | Value                                   | Source  | Comment |
| ----- | -------------- | ---------------- | --------------------------------------- | ------- | ------- |
| Front | R1, R3, R5, R7 | Resistor         | 100kΩ                                   |         | |
| Front | R2, R4, R6, R8 | Resistor         | 200kΩ                                   |         | |
| Front | R9-R12         | Resistor         | 2kΩ                                     |         | Should match potentiometer values. If you are using 50kΩ pots, use 10kΩ resistors here. |
| Back  | R13-R20        | Resistor         | 100kΩ                                   |         | |
| Back  | R20-R24        | Resistor         | 10kΩ                                    |         | Determines LED brightness. Any value between 220Ω-10kΩ might be appropriate depending on which LEDs you have and how bright you want them. Lower resistance values mean more current and brighter LEDs. |
| Front | R25, R26       | Resistor         | 1Ω                                      |         | Determines output impedance |
| Front | RV1-RV4        | Potentiometer    | B10kΩ                                   | [Thonk](https://www.thonk.co.uk/shop/alpha-9mm-pots-dshaft/) | Linear. A larger value is fine, although if the value is too large the response curve might be a little warped. Make sure to match R9-R12 accordingly. |
| Front | SW1            | Button           | TL1105SP (e.g. TL1105SPF250Q) + 1RBLK   | [DigiKey (switch)](https://www.digikey.com/en/products/detail/e-switch/TL1105SPF250Q/271559), [DigiKey (cap)](https://www.digikey.com/en/products/detail/e-switch/1RBLK/271579) | The caps for these switches need to be purchased separately. The caps I use are `#1RBLK`. The switches are available in different actuation forces and materials, so the last part of the part number might be a little different. Sometimes, the switches and caps will be sold together and the cap number is appended to the end of the part number. If you don't want to use these specific switches, any ["6mm tactile switch"](https://www.amazon.com/TWTADE-216Pcs-Momentary-Tactile-Latching/dp/B0C818FLCP) with a standard 4.5mm x 6.5mm mounting pattern should work here. Most of them tend to be very skinny though, so you might want to adjust the faceplate accordingly. |
| Front | J1-J8          | 3.5mm Jack       | THONKICONN (a.k.a PJ398SM or PJ301M-12) | [Thonk](https://www.thonk.co.uk/shop/thonkiconn/) | |
| Front | D1-D11         | LED              | 3mm                                     |         | My design uses 4 colors (amber, blue, red, green) to match the 4 knob colors, but obviously use whatever colors you want. Different colors might have slightly different brightnesses so you could tune R20-R24 to make them uniform. |
| Back  | Q1, Q2         | Transistor       | 2N3904                                  | [DigiKey](https://www.digikey.com/en/products/detail/onsemi/2N3904TA/973944) | |
| Back  | U1             | DAC              | MCP4922-E/P                             | [DigiKey](https://www.digikey.com/en/products/detail/microchip-technology/MCP4922-E-P/716251) | |
| Back  | U2             | Op-amp           | MCP6004                                 | [DigiKey](https://www.digikey.com/en/products/detail/microchip-technology/mcp6004-i-p/523060) | |
| Back  | C1             | Capacitor        | 10uF                                    |         | Power supply noise filtering capacitor |
| Back  | C2, C3         | Capacitor        | 100nF                                   |         | Power supply noise filtering capacitor |
| Back  | A1             | Arduino Nano     | v3.0                                    |         | |
| Both  | J10-J12        | Pin headers      | 1x7, 1x8                                | [Amazon](https://www.amazon.com/gp/product/B074HVBTZ4) | Solder the two boards directly together using the male headers or make them detachable using a male/female pair (recommended). |
| Back  | J13            | IDC connector    | 2x8                                     |         | Eurorack power header. Can use two rows of male pin headers or a shrouded connector (recommended). |
| Back  | J14            | Configuration Jumper |                                     |         | See **Configuration** |

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
