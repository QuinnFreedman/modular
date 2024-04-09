# Output Assembly Instructions

See [general assembly instructions](https://quinnfreedman.github.io/modular/docs/assembly)

Note: this design has 3 PCBs that are stacked into a sandwich. The small one (top) and the one with the large jacks (middle) both face UP, with their components facing the PCB (as indicated by the silkscreen). The last PCB (bottom) faces down, with most of the components mounted on the side facing away from the faceplate. All three are connected together with a single pair of headers at the top and a set of screws or offsets at the bottom.

## Components

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about acquiring parts.

Interactive BOM: [front](https://quinnfreedman.github.io/fm-artifacts/Output/output_pcb_front_interactive_bom.html), [back](https://quinnfreedman.github.io/fm-artifacts/Output/output_pcb_back_interactive_bom.html)

| Board | Reference       | Part             | Value                                   | Source  | Comment |
| ----- | --------------- | ---------------- | --------------------------------------- | ------- | ------- |
| Front | R1, R2          | Resistor         | 100kΩ                                   |         |         |
| Back  | R3, R11         | Resistor         | 2kΩ                                     |         | These resistors overlap the mounting holes. I recommend putting in the standoff first and then putting the resistors over/around the bolt head. |
| Back  | R4-R6, R12-R14  | Resistor         | 1kΩ                                     |         | See R3, R11. |
| Back  | R7-R10, R15-R18 | Resistor         | 10Ω                                     |         | Adds output impedance to headphone output; protects against shorts when plugging in headphones. |
| Back  | R19, R20        | Resistor         | 2kΩ                                     |         | Any value is fine. |
| Back  | R21, R22        | Resistor         | 1kΩ                                     |         | Any value is fine as long as they are 1/2 R19, R20. |
| Back  | R23-R26         | Resistor         | 10kΩ                                    |         | Any value is fine as long as they are all equal. |
| Back  | R27-R30         | Resistor         | 100Ω                                    |         | Output impedance for line level output. |
| Middle| R31, R32        | Resistor         | 22kΩ                                    |         | Controls the decay of the peak detector display. A lower value will make the display follow the audio signal more closely. A higher value will show signal peaks for longer. |
| Middle| R33             | Resistor         | 33kΩ                                    |         | R33-R38 provide the voltage reference for the volume meter. To get an accurate meter, they all must be the exact values given at 1% tolerance. But, if you don't have all the correct values, only the display will be effected; not the audio. Alternatively, if you want to use a different scale for the meters, you could use your own choice of values here. They are arranged as a voltage divider ladder from 12v to 0v. |
| Middle| R34             | Resistor         | 2.2kΩ                                   |         |         |
| Middle| R35             | Resistor         | 1.5kΩ                                   |         |         |
| Middle| R36             | Resistor         | 2.4kΩ                                   |         |         |
| Middle| R37             | Resistor         | 820Ω                                    |         |         |
| Middle| R38             | Resistor         | 470Ω                                    |         |         |
| Middle| R39-R48         | Resistor         | 10kΩ                                    |         | Controls LED brightness for display. Higher values mean dimmer LEDs. |
| Front | C1, C2          | Capacitor        | 150nF                                   |         | Combines with R1-R2 to make a high-pass filter for the input. 100nF is fine here too. |
| Back  | C3, C4          | Capacitor        | 10uF                                    |         | Power supply noise filtering capacitors. |
| Back  | C5-C14, C19-C24 | Capacitor        | 100nF                                   |         | Power supply noise filtering/decoupling capacitors. |
| Middle| C15-C16, C25-C28| Capacitor        | 100nF                                   |         | Power supply noise filtering/decoupling capacitors. |
| Front | C17, C18        | Capacitor        | 100nF                                   |         | Power supply noise filtering/decoupling capacitors. |
| Back  | C29, C30        | Capacitor        | 100pF                                   |         | Op-amp feedback stabilizers. |
| Middle| C31, C32        | Capacitor        | 10uF                                    |         |         |
| Back  | C23, C34        | Capacitor        | 100pF                                   |         | Op-amp feedback stabilizers. |
| Middle| D1, D2          | Diode            | 1N4148                                  |         |         |
| Middle| D3, D8          | LED              | RED (3mm)                               | [Amazon](https://www.amazon.com/gp/product/B077X96HDR/) | Must have long enough leads to reach from middle PCB to faceplate (~27mm) |
| Middle| D4, D9          | LED              | AMBER (3mm)                             |         |         |
| Middle| D5-D7, D10-D12  | LED              | GREEN (3mm)                             |         |         |
| Front | J1, J2          | 3.5mm Jack       | THONKICONN (a.k.a PJ398SM or PJ301M-12) | [Thonk](https://www.thonk.co.uk/shop/thonkiconn/) | |
| Front | J3-J5           | 1/4in Jack       | ACJS-MN-3                               | [Mouser](https://www.mouser.com/ProductDetail/Amphenol-Audio/ACJS-MN-3?qs=c9RBuMmXG6ItVexVLzhiSw%3D%3D) | Only 3 poles are used but there are 5 holes on the PCB so the switched versions (ACJS-MN-3S or ACJS-MN-5) will work as well. ACJS-MV-3 is exactly the same except that it has a star washer to help make contact with the faceplate. That will work too (as will ACJS-MV-3S or ACJS-MV-5). |
| Back  | J6              | IDC connector    | 2x5                                     | [DigiKey](https://www.digikey.com/en/products/detail/on-shore-technology-inc/302-S101/2178422) | Eurorack power header. Can use two rows of male pin headers or a shrouded connector (recommended). |
| Back  | J7              | Unused           | -                                       |         | Expansion connector for future modules. |
| All   | J11-J23         | Stacking Pin Headers | 1x10                                | [Amazon](https://www.amazon.com/dp/B0B3XBYL3J) | All 3 PCBs are connected by a single set of headers. There are a few ways to do this. You could use a stacking male/female header on the middle and use standard male/female headers on the top and bottom, use double-sided male headers on the middle and standard female on the top and bottom, or just use extra-long male headers and solder everything in place. The only thing that matters is that the offset between the front and middle PCBs is the right distance to align the large and small jacks. |
| Front | RV1, RV2        | Potentiometer    | B50kΩ                                   | [Thonk](https://www.thonk.co.uk/shop/alpha-9mm-pots-dshaft/) | Linear. Any value is fine. |
| Front | U1              | Op-amp           | TL072                                   | [DigiKey](https://www.digikey.com/en/products/filter/instrumentation-op-amps-buffer-amps/687?s=N4IgjCBcoGwJxVAYygMwIYBsDOBTANCAPZQDaIALGGABxwDsIAuoQA4AuUIAyuwE4BLAHYBzEAF9CAWgTQQKSPwCuBYmXDNJIKQCZE8qMtUlI5AKwhCCJuK17TIdpgAM9PSxAx9AgCZcpYM4QbJyQIJaOAJ6suFzo2Ci2QA) | |
| Back  | U2, U7          | Op-amp           | TL074                                   | [DigiKey](https://www.digikey.com/en/products/detail/texas-instruments/TL074BCN/378416) | |
| Back  | U3-U6, U8-U9    | Op-amp           | NE5532P                                 | [Mouser](https://www.mouser.com/ProductDetail/595-NE5532P) | The NE5532P is a low-noise, high output-drive Op-amp, used here for driving the headphones and line outputs. The NE5532AP is an extra low-noise version, although it's probably not worth the cost. The SA5532 is identical except it has a higher temperature range, which isn't necessary. If you have a particular op-amp you prefer, you can use it here instead (as long as it follows the standard TL072-style pinout). A TL072 should work fine, although it might not have the best distortion characteristics or lifespan depending on what load you are hooking it up to. U5 and U6 are connected in parallel for extra driving power and distortion resistance on the headphone output. You can just leave them out if you don't need the extra power. |
| Middle| U10-U12         | Op-amp           | TL074                                   | [DigiKey](https://www.digikey.com/en/products/detail/texas-instruments/TL074BCN/378416) | |

