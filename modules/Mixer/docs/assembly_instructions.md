# Mixer Assembly Instructions

See [general assembly instructions](https://quinnfreedman.github.io/modular/docs/assembly)

## Components

All parts are available on Tayda ([cart link](https://www.taydaelectronics.com/savecartpro/index/savenewquote/qid/07540677724), [quick-order CSV](https://freemodular.org/modules/Mixer/fm_mixer_tayda_bom.csv)).

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about acquiring parts.

[Interactive BOM](https://quinnfreedman.github.io/fm-artifacts/Mixer/mixer_pcb_interactive_bom.html)

| Reference | Part             | Value                                   | Source  | Comment |
| --------- | ---------------- | --------------------------------------- | ------- | ------- |
| R1-R9     | Resistor         | 100kΩ                                   | [Tayda](https://www.taydaelectronics.com/10-x-resistor-100k-ohm-1-4w-1-metal-film-pkg-of-10.html) | Any value is fine for these, as long as they are all the same, and are significantly higher than the output impedance of the sources you are mixing. |
| R10       | Resistor         | 1kΩ                                     | [Tayda](https://www.taydaelectronics.com/10-x-resistor-1k-ohm-1-4w-1-metal-film-pkg-of-10.html) | Determines output impedance. Any value is fine. |
| R11       | Resistor         | 4.7kΩ                                   | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/10-x-resistor-4-7k-ohm-1-4w-1-metal-film-pkg-of-10.html) | Determines LED brightness. You may want to use a different value if you have different LEDs. A lower value means less resistance and brighter LEDs. |
| C1,C2     | Capacitor        | 100nF                                   | [Tayda](https://www.taydaelectronics.com/a-553-0-1uf-50v-ceramic-disc-capacitor-pkg-of-10.html) | **[Optional]** Power supply noise filtering/decoupling capacitor |
| C3,C4     | Capacitor        | 33pF                                    | [Tayda](https://www.taydaelectronics.com/10-x-33pf-50v-ceramic-disc-capacitor-pkg-of-10.html) | **[Optional]** Amp stabilization. Anything in the 100pF-33pF range is probably fine. |
| C5,C6     | Capacitor        | 10uF                                    | [Tayda](https://www.taydaelectronics.com/10uf-16v-85c-radial-electrolytic-capacitor.html) | **[Optional]** Power supply noise filtering capacitor |
| RV1-RV4   | Potentiometer    | A100kΩ                                  | [Tayda](https://www.taydaelectronics.com/potentiometer-variable-resistors/rotary-potentiometer/100k-ohm-logarithmic-taper-potentiometer-d-shaft-pcb-9mm.html), [Thonk](https://www.thonk.co.uk/shop/alpha-9mm-pots-dshaft/) | Logarithmic is best if you primarily want to mix audio. Any value should work for these as long as they are significantly higher than the output impedance of the sources you are mixing. |
| SW1-SW5   | Switch           | TAIWAY 200CWMSP1T3B4M2                  | [Tayda](https://www.taydaelectronics.com/sub-mini-toggle-switch-2m-series-spdt-on-on-pcb-pins.html), [Thonk](https://www.thonk.co.uk/shop/sub-mini-toggle-switches/), [Love My Switches](https://lovemyswitches.com/taiway-sub-mini-spdt-on-on-switch-pcb-mount-long-shaft/) | Sub-mini SPDT ON-ON |
| J1-J6     | Jack Socket      | PJ301M-12                               | [Tayda](https://www.taydaelectronics.com/pj-3001f-3-5-mm-mono-phone-jack.html), [Thonk](https://www.thonk.co.uk/shop/thonkiconn/) | |
| J7        | Power header     | IDC male 2x5                            | [Tayda](https://www.taydaelectronics.com/10-pin-box-header-connector-2-54mm.html) | |
| J8, J9    | Jumper headers   |                                         | Tayda [1](https://www.taydaelectronics.com/connectors-sockets/wafer-housing-crimp-terminal/9376-dup-housing-connector-2-54mm-2-pins.html), [2](https://www.taydaelectronics.com/crimp-terminal-connector-xh-2-54mm.html), [3](https://www.taydaelectronics.com/connectors-sockets/wafer-housing-crimp-terminal/xh-connectors/2-pins-jst-xh-2-54-male-connector-straight-180-degree.html) | **[Optional]** Chain multiple modules together by connecting OUT to IN. |
| D1        | LED              | 3mm                                     | [Tayda](https://www.taydaelectronics.com/led-3mm-red.html) | Indicates output level. Optionally, you could use a bidirectional/bipolar LED here. Using a normal LED as shown in the design, with anode on top, will only illuminate when the mixer is outputting a positive voltage. |
| U1        | Op-amp           | TL072                                   | [Tayda](https://www.taydaelectronics.com/tl072-low-noise-j-fet-dual-op-amp-ic.html) | |

