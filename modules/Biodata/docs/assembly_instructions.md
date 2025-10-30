# Biodata Assembly Instructions

## Components

**Most** parts are available on Tayda ([quick-order CSV](https://freemodular.org/modules/Biodata/fm_biodata_tayda_bom.csv)).

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about sourcing parts.

Interactive BOM: [front](https://quinnfreedman.github.io/fm-artifacts/Biodata/biodata_pcb_front_interactive_bom.html), [back](https://quinnfreedman.github.io/fm-artifacts/Biodata/biodata_pcb_back_interactive_bom.html)

|    | Board | Reference                  | Part             | Value                                   | Source  | Comment |
| -- | ----- | -------------------------- | ---------------- | --------------------------------------- | ------- | ------- |
|    | Back  | R1, R6-R7, R13-R15         | Resistor         | 1kΩ                                     | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/10-x-resistor-1k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
|    | Front | R16-R17                    | Resistor         | 1kΩ                                     | - | |
|    | Back  | R2, R3                     | Resistor         | 1MΩ                                     | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/10-x-resistor-1m-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
|    | Back  | R4-R5                      | Resistor         | 100kΩ                                   | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/10-x-resistor-100k-ohm-1-4w-1-metal-film-pkg-of-10.html) |  |
|    | Back  | R8, R11-R12                | Resistor         | 10kΩ                                    | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/10-x-resistor-10k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
|    | Back  | R9-R10                     | Resistor         | 220kΩ                                   | [Tayda](https://www.taydaelectronics.com/resistor-220k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
|    | Back  | C1                         | Capacitor        | 10nF                                    | [Tayda](https://www.taydaelectronics.com/10nf-0-01uf-100v-5-jfa-mylar-film-capacitors.html) | |
|    | Back  | C2                         | Capacitor        | 1nF                                     | [Tayda](https://www.taydaelectronics.com/capacitors/polyester-mylar-film-capacitors/1nf-0-001uf-100v-5-mylar-film-capacitors.html) | |
|    | Back  | C3-C4, C7-C9               | Capacitor        | 100nF                                   | [Tayda](https://www.taydaelectronics.com/100nf-50v-multilayer-monolithic-ceramic-capacitor-2-54mm-vishay.html) | |
|    | Back  | C10                        | Capacitor        | 100pF                                   | [Tayda](https://www.taydaelectronics.com/10-x-100pf-50v-ceramic-disc-capacitor-pkg-of-10.html) | |
|    | Back  | C6-C12                     | Capacitor        | 10uF                                    | [Tayda](https://www.taydaelectronics.com/10uf-16v-85c-radial-electrolytic-capacitor.html) | |
|    | Front | RV1                        | Potentiometer    | B50k                                    | [Tayda](https://www.taydaelectronics.com/potentiometer-variable-resistors/rotary-potentiometer/linear/50k-ohm-linear-taper-potentiometer-d-shaft-pcb-9mm.html) | Any value 1k-100k is fine |
|    | Front | RV2-RV3                    | Potentiometer    | B50k                                    | [Tayda](https://www.taydaelectronics.com/potentiometer-variable-resistors/rotary-potentiometer/linear/tayda-10k-ohm-linear-taper-potentiometer-spline-shaft-pcb-mount-25mm.html) | No-knob splined-shaft potentiometers. Any value 1k-100k is fine |
|    | Front | J1-J6                      | 3.5mm Jack       | PJ398SM or PJ301M-12                    | [Tayda](https://www.taydaelectronics.com/pj-3001f-3-5-mm-mono-phone-jack.html) | |
|    | Back  | J7                         | IDC connector    | 2x8                                     | [Tayda](https://www.taydaelectronics.com/16-pin-box-header-connector-2-54mm.html) | Eurorack power header |
|    | Both  | J14-J19                    | Pin headers      | 1x6, 1x7                                | Tayda ([Male](https://www.taydaelectronics.com/40-pin-2-54-mm-single-row-pin-header-strip.html), [Female](https://www.taydaelectronics.com/40-pin-2-54-mm-single-row-female-pin-header.html)), [Amazon](https://www.amazon.com/gp/product/B074HVBTZ4) | Cut headers down to size. Attach the boards together using a male/female header pair for each connection. |
|    | Back  | Q1-Q2                      | NPN Transistor   | 2N3904                                  | [Tayda](https://www.taydaelectronics.com/2n3904-npn-general-propose-transistor.html) | |
|    | Front | D1                         | LED              | 5mm                                     | [Tayda](https://www.taydaelectronics.com/led-5mm-green.html) | |
|    | Back  | U1                         | Timer/oscillator | NE555P                                  | [Tayda](https://www.taydaelectronics.com/ne555-ic-555-texas-timer.html) | |
|    | Back  | U2                         | Op-amp           | TL072                                   | [Tayda](https://www.taydaelectronics.com/tl072-low-noise-j-fet-dual-op-amp-ic.html) | TL082 is probably fine too |
| 🔴 | Back  | U3                         | Power isolator   | TEA 1-0505                              | [DigiKey](https://www.digikey.com/short/4jjrfr39), [Mouser](https://mou.sr/3PANyDz) | This electrically isolates the power sent over the probes from the rest of the Eurorack case. This is intended to reduce 60Hz hum or other electrical interference form being picked up as part of the signal. If you don't feel this is necessary, you can short pins 1 and 3 and pins 2 and 4 to bypass this component. |
|    | Back  | U4                         | Signal opto-isolator | PC817X1NSZ9F                            | [Tayda](https://www.taydaelectronics.com/pc817a-pc817-photocoupler-phototransistor-1-channel-output-ic.html) | Electrically isolates the 555 galvanometer signal from the main power supply (see above). Again, could be omitted and bypassed by shorting pins 1 and 4. Omit R15 as well in this case. |
| 🔴 | Back  | U5                         | DAC              | MCP4922-E/P                             | [DigiKey](https://www.digikey.com/en/products/detail/microchip-technology/MCP4922-E-P/716251), [Mouser](https://mou.sr/4cwtePf) | |
|    | Back  | A1                         | Arduino Nano     | v3.0                                    | [Tayda](https://www.taydaelectronics.com/type-c-nano-3-0-controller-compatible-with-arduino-nano.html) | |
| 🔴 | Both  | -                          | Mounting screw   | M2.5x11mm standoff or bolt              |  | Holds the two PCBs together. |

🔴 = Missing from Tayda BOM

