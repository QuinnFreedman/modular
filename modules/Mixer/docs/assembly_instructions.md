# Mixer Assembly Instructions

See [general assembly instructions](https://quinnfreedman.github.io/modular/docs/assembly)

## Components

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about acquiring parts.

[Interactive BOM](https://quinnfreedman.github.io/fm-artifacts/Mixer/mixer_pcb_interactive_bom.html)

| Reference | Part             | Value                                   | Source  | Comment |
| --------- | ---------------- | --------------------------------------- | ------- | ------- |
| R1-R9     | Resistor         | 100kΩ                                   |         | Any value is fine for these, as long as they are all the same, and are significantly higher than the output impedance of the sources you are mixing. |
| R10       | Resistor         | 1kΩ                                     |         | Determines output impedance. Any value is fine. |
| R11       | Resistor         | 4.7kΩ                                   |         | Determines LED brightness. You may want to use a different value if you have different LEDs. A lower value means less resistance and brighter LEDs. |
| C1,C2     | Capacitor        | 100nF                                   |         | **[Optional]** Power supply noise filtering capacitor |
| C3,C4     | Capacitor        | 33pF                                    |         | **[Optional]** Amp stabilization. Anything in the 100pF-33pF range is probably fine. |
| C5,C6     | Capacitor        | 10uF                                    |         | **[Optional]** Power supply noise filtering capacitor |
| RV1-RV4   | Potentiometer    | A100kΩ                                  | [Thonk](https://www.thonk.co.uk/shop/alpha-9mm-pots-dshaft/) | Logarithmic is best if you primarily want to mix audio. Any value should work for these as long as they are significantly higher than the output impedance of the sources you are mixing.|
| SW1-SW5   | Switch           | TAIWAY 200CWMSP1T3B4M2                  | [Thonk](https://www.thonk.co.uk/shop/sub-mini-toggle-switches/), [Love My Switches](https://lovemyswitches.com/taiway-sub-mini-spdt-on-on-switch-pcb-mount-long-shaft/) | SPDT ON-ON |
| J1-J6     | Jack Socket      | PJ301M-12                               | [Thonk](https://www.thonk.co.uk/shop/thonkiconn/) | |
| J7        | Power header     | IDC male 2x5                            |         | |
| J8, J9    | Jumper headers   |                                         |         | **[Optional]** Chain multiple modules together by connecting OUT to IN. |
| D1        | LED              | 3mm                                     |         | Indicates output level. Optionally, you could use a bidirectional/bipolar LED here. Using a normal LED as shown in the design, with anode on top, will only illuminate when the mixer is outputting a positive voltage. |
| U1        | Op-amp           | TL072                                   |         | |

