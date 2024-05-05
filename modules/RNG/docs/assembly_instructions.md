# RNG Assembly Instructions

See [general assembly instructions](https://quinnfreedman.github.io/modular/docs/assembly)

## Components

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about acquiring parts.

Interactive BOM: [front](https://quinnfreedman.github.io/fm-artifacts/RNG/rng_pcb_front_interactive_bom.html), [back](https://quinnfreedman.github.io/fm-artifacts/RNG/rng_pcb_back_interactive_bom.html)

| Board | Reference | Part             | Value                                   | Source  | Comment |
| ----- | --------- | ---------------- | --------------------------------------- | ------- | ------- |
| Front | R1        | Resistor         | 100kΩ                                   |         |         |
| Front | R2, R3    | Resistor         | 1kΩ                                     |         | Determines output impedance. Any value is fine. |
| Front | R4-R7     | Resistor         | 10kΩ                                    |         | Determines LED brightness. You may want to use a different value if you have different LEDs. A lower value means less resistance and brighter LEDs |
| Front | RV1-RV3   | Potentiometer    | B50kΩ                                   | [Thonk](https://www.thonk.co.uk/shop/alpha-9mm-pots-dshaft/) | Linear. Any value is fine. |
| Front | D1-D11    | LED              | 3mm                                     |         | Any standard 3mm LED will work here. |
| Front | SW1       | Rotary Encoder   | EC11 series                             | [Amazon](https://www.amazon.com/dp/B07D3DF8TK) | |
| Front | SW2, SW3  | Switch           | TAIWAY 200CWMSP1T3B4M2                  | [Love My Switches](https://lovemyswitches.com/taiway-sub-mini-spdt-on-on-switch-pcb-mount-long-shaft/), [Thonk](https://www.thonk.co.uk/shop/sub-mini-toggle-switches/) | SPDT ON-ON |
| Front | J1-J6     | 3.5mm Jack       | THONKICONN (a.k.a PJ398SM or PJ301M-12) | [Thonk](https://www.thonk.co.uk/shop/thonkiconn/) | |
| Both  | J8-J10    | Pin headers      | 2x3, 1x8, 1x8                           | [Amazon](https://www.amazon.com/gp/product/B074HVBTZ4) | Solder the two boards directly together using the male headers or (recommended) make them detachable using a male/female pair. |
| Back  | J7        | -                | -                                       |         | Not used. Expansion points for future features |
| Back  | J12       | IDC connector    | 2x8                                     |         | Eurorack power header. Can use two rows of male pin headers or (recommended) a shrouded connector. |
| Back  | R8-R11, R14-R15 | Resistor   | 100kΩ                                   |         |         |
| Back  | R12, R13, R16   | Resistor   | 50kΩ                                    |         |         |
| Back  | R17       | Resistor         | 1kΩ                                     |         | Determines output impedance. Any value is fine. |
| Back  | R18       | Resistor         | 15kΩ                                    |         | Controls the LED brightness for the 7-LED display. Unlike the bottom LEDs, which are in series with R4-R7 (@5v), this resistor is just used as a current refference (@5v) by the TLC5940. To match the current across all the LEDs (and therefore the brightness), R18 is calculated by R18 = 5 / ((5 - V_LED) / R4), where V_LED is the voltage drop across one of the LEDs (at the current they will recieve based of R4-R7 at 5v). This can be measured or will probably be in datasheet. Then round to the nearrest available LED value. It doesn't have to be exact. |
| Back  | R19       | Resistor         | 15kΩ                                    |         | Controls the available current at the -5v ref regulator. You could probably go a little higher to be more power efficient, but if the -5v voltage sags you can decrease the value. |
| Back  | C1-C5     | Capacitor        | 100nF                                   |         | **Optional.** Power supply noise filtering capacitors |
| Back  | C6-C8     | Capacitor        | 10uF                                    |         | **Optional.** Power supply noise filtering capacitors |
| Back  | C6-C8     | Capacitor        | 10uF                                    |         | **Optional.** Power supply noise filtering capacitors |
| Back  | A1        | Arduino Nano     | v3.0                                    |         | |
| Back  | U1        | LED driver       | TLC5940NT                               | [DigiKey](https://www.digikey.com/en/products/detail/texas-instruments/TLC5940NT/716896) | |
| Back  | U2        | DAC              | MCP4922-E/P                             | [DigiKey](https://www.digikey.com/en/products/detail/microchip-technology/MCP4922-E-P/716251) | |
| Back  | U3        | Op-amp           | MCP6004                                 | [DigiKey](https://www.digikey.com/en/products/detail/microchip-technology/mcp6004-i-p/523060) | |
| Back  | U4        | Op-amp           | TL072                                   | [DigiKey](https://www.digikey.com/en/products/filter/instrumentation-op-amps-buffer-amps/687?s=N4IgjCBcoGwJxVAYygMwIYBsDOBTANCAPZQDaIALGGABxwDsIAuoQA4AuUIAyuwE4BLAHYBzEAF9CAWgTQQKSPwCuBYmXDNJIKQCZE8qMtUlI5AKwhCCJuK17TIdpgAM9PSxAx9AgCZcpYM4QbJyQIJaOAJ6suFzo2Ci2QA) | |
| Back  | U5        | Voltage regulator| LM4040BIZ-5                             | [DigiKey](https://www.digikey.com/en/products/detail/rochester-electronics-llc/LM4040BIZ-5-0/12603438) | 5v, TO92-3 package |

