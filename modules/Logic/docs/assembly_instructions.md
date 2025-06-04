# ALU Assembly Instructions

The ALU is mostly built designed with surface-mount components (it was the only wat to fit this circuit in such a small HP footprint). These parts are relatively large (0805-size or equivalent) so they are definitely possible to solder by hand. But, it is probably much more cost effective to just have the fabrication house making the PCBs also assemble them. The KiCad files have fields for JLC part numbers, so you can easily export the needed materials to have JLCPCB manufacture the whole board thing. But equivalent parts should be available at any fab service.

## Components

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about sourcing parts.

Interactive BOM: [front](https://quinnfreedman.github.io/fm-artifacts/Logic/logic_pcb_front_interactive_bom.html), [back](https://quinnfreedman.github.io/fm-artifacts/Logic/logic_pcb_back_interactive_bom.html)

### Through-hole components

| Board | Reference                  | Part             | Value                                   | Source  | Comment |
| ----- | -------------------------- | ---------------- | --------------------------------------- | ------- | ------- |
| Front | J1-J12                     | 3.5mm Jack       | THONKICONN (a.k.a PJ398SM or PJ301M-12) | [Tayda](https://www.taydaelectronics.com/pj-3001f-3-5-mm-mono-phone-jack.html), [Thonk](https://www.thonk.co.uk/shop/thonkiconn/) | |
| Both  | J14-J15, J16-J17           | Pin headers      | 1x7, 1x6                                | Tayda ([Male](https://www.taydaelectronics.com/40-pin-2-54-mm-single-row-pin-header-strip.html), [Female](https://www.taydaelectronics.com/40-pin-2-54-mm-single-row-female-pin-header.html)), [Amazon](https://www.amazon.com/gp/product/B074HVBTZ4) | Cut headers down to size. Solder the two boards directly together using the male headers or make them detachable using a male/female pair. |
| Back  | J13                        | IDC connector    | 2x8                                     | [Tayda](https://www.taydaelectronics.com/16-pin-box-header-connector-2-54mm.html) | Eurorack power header |

### Surface-mount components

| Board | Reference                          | Part              | Value           | Footprint | JLCPCB part number |
| ----- | ---------------------------------- | ----------------- | --------------- | --------- | ------------------ |
| Back  | C1-C4                              | Capacitor         | 330pF           | 0805      | C376974            |
| Back  | C8-C16                             | Capacitor         | 100nF           | 0805      | C29926             |
| Back  | C5-C6                              | Capacitor         | 10uF            | 4x5.4     | C3343              |
| Back  | R14, R17-R18, R27-R29              | Resistor          | 1kΩ             | 0805      | C17513             |
| Back  | R30                                | Resistor          | 3.3kΩ           | 0805      | C26010             |
| Back  | R3-R5, R8                          | Resistor          | 5.1kΩ           | 0805      | C27834             |
| Back  | R1-R2, R7-R7, R9-R13, R15, R19-R26 | Resistor          | 10kΩ            | 0805      | C17414             |
| Back  | R16                                | Resistor          | 100kΩ           | 0805      | C57246             |
| Back  | D1-D3                              | Diode             | 1N4148          | SOD-123   | C5443965           |
| Back  | U1-U2                              | NAND gates        | CD74HCT132E     | SOIC-14   | C484714            |
| Back  | U3-U4                              | Op-amp            | LM324           | SOIC-14   | C7943              |
| Back  | U6                                 | Voltage regulator | LM4040C50FTA    | SOT-23    | C156291            |
| Back  | U5                                 | Op-amp            | TL072           | SOP-8     | C5310783           |

