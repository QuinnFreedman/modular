# Clock Assembly Instructions

## Components

Interactive BOM: [front](https://raw.githubusercontent.com/QuinnFreedman/fm-artifacts/main/Clock/clock_front_pcb_interactive_bom.html), [back](https://raw.githubusercontent.com/QuinnFreedman/fm-artifacts/main/Clock/clock_back_pcb_interactive_bom.html)


| Board | Reference | Part             | Value                                   | Source  | Comment |
| ----- | --------- | ---------------- | --------------------------------------- | ------- | ------- |
| Front | R1-R8     | Resistor         | 1kΩ                                     |         | Determines output impedance. Any value is fine. |
| Front | R1-R8     | Resistor         | 5.1kΩ                                   |         | Determines LED brightness. You may want to use a different value if you have different LEDs. A lower value means less resistance and brighter LEDs |
| Front | D1-D8     | LED              | 3mm                                     |         | Any standard 3mm LED will work here. |
| Front | SW1       | Rotary Encoder   | EC11 series                             | [amazon](https://www.amazon.com/dp/B07D3DF8TK) | |
| Front | SW2       | Push button      | TL1105SPF250Q-1RBLK                     | button: [DigiKey](https://www.digikey.com/en/products/detail/e-switch/TL1105SPF250Q/271559), cap: [DigiKey](https://www.digikey.com/en/products/detail/e-switch/1RBLK/271579) | I might want to switch this out for a larger button in a later revision. |
| Front | Screen    | OLED display     | SSD1306 SPI                             | [ebay](https://www.ebay.com/itm/373647815247) | The underlying display controller is the SSD1306. There are many ebay/amazon sellers that make modules based on it. Make sure to get one that uses the SPI protocol (6 or 7 total pins) instead of the I2C protocol (4 pins). The pins may be labelled differently or be in a different order on different modules. Just connect them to the corresponding holes on the PCB. The module also may or may not contain a 7th RESET pin. Either way should be fine. Different modules may also vary in the layout of their mounting holes. The faceplate is designed for a square bolt pattern of 0.9in, but it could be easily modified to fit a different screen. You can connect the screen to the PCB by soldering each wire individually or by soldering on some kind of 7-pin plug. |
| Front | J1-J8     | 3.5mm Jack       | THONKICONN (a.k.a PJ398SM or PJ301M-12) | [thonk](https://www.thonk.co.uk/shop/thonkiconn/) | |
| Front | J9-J11    | Pin headers      | 1x4, 1x4, 1x11                          | [amazon](https://www.amazon.com/gp/product/B074HVBTZ4) | Solder the two boards directly together using the male headers or (recommended) make them detachable using a male/female pair. |
| Back  | C1        | Capacitor        | 100nF                                   |         | **Optional.** Power supply noise filtering capacitor |
| Back  | C2        | Capacitor        | 10uF                                    |         | **Optional.** Power supply noise filtering capacitor |
| Back  | A1        | Arduino Nano     | v3.0                                    |         | |
| Back  | J9-J11    | Pin headers      | 1x4, 1x4, 1x11                          |         | Match corresponding headers on front PCB |
| Back  | J12       | IDC connector    | 2x8                                     |         | Eurorack power header. Can use two rows of male pin headers or (recommended) a shrouded connector. |
| Back  | J13       | -                | -                                       |         | Not used. Expansion points for future features |

