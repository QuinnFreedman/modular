# Drift Assembly Instructions

See [general assembly instructions](https://quinnfreedman.github.io/modular/docs/assembly)

## Components

**Most** parts are available on Tayda ([quick-order CSV](https://freemodular.org/modules/Drift/fm_drift_tayda_bom.csv)).

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about acquiring parts.

Interactive BOM: [front](https://quinnfreedman.github.io/fm-artifacts/Drift/drift_pcb_front_interactive_bom.html), [back](https://quinnfreedman.github.io/fm-artifacts/Drift/drift_cb_back_pinteractive_bom.html)

|    | Board | Reference | Part             | Value                                   | Source  | Comment |
| -- | ----- | --------- | ---------------- | --------------------------------------- | ------- | ------- |
|    | Back  | R1        | Resistor         | 1kΩ                                     | [Tayda](https://www.taydaelectronics.com/10-x-resistor-1k-ohm-1-4w-1-metal-film-pkg-of-10.html) | Determines output impedance. Any value is fine. |
|    | Back  | R2-R4     | Resistor         | 10kΩ                                    | [Tayda](https://www.taydaelectronics.com/10-x-resistor-10k-ohm-1-4w-1-metal-film-pkg-of-10.html) | R4 should be 1/2 RV3. |
|    | Back  | R5        | Resistor         | 1kΩ                                     |         | Controls LED brightness. |
|    | Back  | R6, R7    | Resistor         | 100kΩ                                   | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/10-x-resistor-100k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
|    | Back  | C1-C4     | Capacitor        | 100nF                                   | [Tayda](https://www.taydaelectronics.com/capacitors/ceramic-disc-capacitors/a-553-0-1uf-50v-ceramic-disc-capacitor-pkg-of-10.html) | Power filtering and decoupling |
|    | Back  | C5-C7     | Capacitor        | 10uF                                    | [Tayda](https://www.taydaelectronics.com/10uf-16v-85c-radial-electrolytic-capacitor.html) | Power supply noise filtering capacitor |
|    | Back  | C8        | Capacitor        | 4.7nF                                   | [Tayda](https://www.taydaelectronics.com/capacitors/ceramic-disc-capacitors/a-553-0-1uf-50v-ceramic-disc-capacitor-pkg-of-10.html) | **Optional:** This capacitor creates a 1.25kHz lowpass filter with R3 which is applied to the output to smooth out the relatively slow digital sample rate of this module. You can try different values for a smoother signal or leave this out entirely if you don't mind some high-frequency artifacts. |
|    | Front | D1        | LED              | 3mm diffuse orange                      | [Tayda](https://www.taydaelectronics.com/leds/round-leds/5mm-leds/led-5mm-yellow.html) | Any standard 5mm LED will work here. |
|    | Front | J1-J3     | 3.5mm Jack       | THONKICONN (a.k.a PJ398SM or PJ301M-12) | [Tayda](https://www.taydaelectronics.com/pj-3001f-3-5-mm-mono-phone-jack.html) | |
|    | Both  | J4-J7     | Pin headers      | 1x4, 1x6                                | Tayda ([Male](https://www.taydaelectronics.com/40-pin-2-54-mm-single-row-pin-header-strip.html), [Female](https://www.taydaelectronics.com/40-pin-2-54-mm-single-row-female-pin-header.html)), [Amazon](https://www.amazon.com/gp/product/B074HVBTZ4) | Solder the two boards directly together using the male headers or make them detachable using a male/female pair. |
| 🔴 | Both  | -         | Mounting screw   | M2                                      | [McMaster-Carr](https://www.mcmaster.com/products/screws/socket-head-screws~/system-of-measurement~metric/thread-size~m2/) | **Optional:** add an M2 screw or standoff to hold the two PCBs firmly together. |
|    | Back  | J9        | IDC connector    | 2x8                                     | [Tayda](https://www.taydaelectronics.com/16-pin-box-header-connector-2-54mm.html) | Eurorack power header. Can use two rows of male pin headers or a shrouded connector (recommended). |
|    | Front | RV1-RV3   | Arduino Nano     | B50kΩ                                   | [Tayda](https://www.taydaelectronics.com/potentiometer-variable-resistors/rotary-potentiometer/linear/50k-ohm-linear-taper-potentiometer-d-shaft-pcb-9mm.html) | Any value is fine. Just match R4 accordingly. |
|    | Back  | A1        | Arduino Nano     | v3.0                                    | [Tayda](https://www.taydaelectronics.com/type-c-nano-3-0-controller-compatible-with-arduino-nano.html) | |
| 🔴 | Back  | U1        | DAC              | MCP4922-E/P                             | [DigiKey](https://www.digikey.com/en/products/detail/microchip-technology/MCP4922-E-P/716251), [Mouser](https://mou.sr/4cwtePf) | |
|    | Back  | U2        | Op-amp           | TL072                                   | [Tayda](https://www.taydaelectronics.com/tl072-low-noise-j-fet-dual-op-amp-ic.html) | TL082 is probably fine too |
|    | Back  | U3        | Op-amp           | MCP6002                                 | [Tayda](https://www.taydaelectronics.com/mcp6002-single-supply-cmos-ic.html), [Mouser](https://mou.sr/4cwtePf) | |
|    | Back  | SW1       | DIP switches     | 1x2                                     | [Tayda](https://www.taydaelectronics.com/black-dip-switch-2-positions-gold-plated-contacts-top-actuated.html) | **Optional:** configuration switches to select noise algorithm. Leave unconnected for the default Perlin noise. You can solder a wire to bridge pairs 1 and/or 2 to select a different mode, or use pair of switches if you want to be able to change it later. See the manual for how to select algorithms. |

🔴 = Missing from Tayda BOM
