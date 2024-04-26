# Offset/Atten Assembly Instructions

See [general assembly instructions](https://quinnfreedman.github.io/modular/docs/assembly)

## Components

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about acquiring parts.

Interactive BOM: [front](https://quinnfreedman.github.io/fm-artifacts/OffsetAtten/rng_pcb_front_interactive_bom.html), [back](https://quinnfreedman.github.io/fm-artifacts/OffsetAtten/rng_pcb_back_interactive_bom.html)

| Board | Reference | Part             | Value                                   | Source  | Comment |
| ----- | --------- | ---------------- | --------------------------------------- | ------- | ------- |
| Front | R1-R4     | Resistor         | 33kΩ                                    |         | Should be ~5/7 times the resistance of R2/R4. |
| Front | RV1-RV4   | Potentiometer    | B50kΩ                                   | [Thonk](https://www.thonk.co.uk/shop/alpha-9mm-pots-dshaft/) | Linear. Any value is fine. Determines the input impedance of the module, so a higher value will mean a better attenuation range for some inputs (100k might actually be better). |
| Front | J1-J4     | 3.5mm Jack       | THONKICONN (a.k.a PJ398SM or PJ301M-12) | [Thonk](https://www.thonk.co.uk/shop/thonkiconn/) | |
| Front | D1-D11    | LED              | 3mm bipolar                             | [ebay](https://www.ebay.com/itm/133972966618) | This is a bipolar/bi-color LED. That means it is essentially two LEDs wired in opposite directions in the same package. Reversing the connection reverses the colors. When the module is outputting a positive voltage, the current will flow "backwards" through the LED, i.e. from the square "cathode" hole towards the top of the module to the round "anode" hole towards the bottom of the module. When the module is outputting a negative voltage, the current will be reversed. Depending on which LEDs you are using and which colors you want, you might need to solder the LED in the reverse direction. It's probably best to just test the LEDs with alligator clips or something similar before soldering. |
| Back  | R5-R8, R11-R14 | Resistor    | 10kΩ                                    |         | Any value 10k-100k is probably fine as long as they all match. |
| Back  | R9, R16   | Resistor         | 10kΩ                                    |         | Determines LED brightness. Any value between 220Ω-10kΩ might be appropriate depending on which LEDs you have and how bright you want them. |
| Back  | R10, R15  | Resistor         | 1kΩ                                     |         | Determines output impedance |
| Back  | C1-C2     | Capacitor        | 10uF                                    |         | Power supply noise filtering capacitors |
| Back  | C3-C6     | Capacitor        | 100nF                                   |         | Power supply noise filtering capacitors |
| Back  | C7-C10    | Capacitor        | 330pF                                   |         | Stabilizing cap for op-amps. Depending on the model of op-amps used, these probably aren't strictly necessary. 100pF should also be fine. |
| Back  | U1, U2    | Op-amp           | LM324N                                  | [DigiKey](https://www.digikey.com/en/products/detail/texas-instruments/LM324N/277627) | Any quad op-amp would probably work here, as long as it follows the standard pinout and can handle 24V power supply. A TL074 would work fine, but it can't handle rail-to-rail inputs, so it might give unexpected behavior if the input or output approaches -12V. Therefore, it is better to use a rail-to-rail-capable op-amp like the LM324. |
| Both  | J5-J8     | Pin headers      | 1x7, 1x8                                | [Amazon](https://www.amazon.com/gp/product/B074HVBTZ4) | Solder the two boards directly together using the male headers or make them detachable using a male/female pair (recommended). |
| Back  | J12       | IDC connector    | 2x8                                     |         | Eurorack power header. Can use two rows of male pin headers or (recommended) a shrouded connector. |

