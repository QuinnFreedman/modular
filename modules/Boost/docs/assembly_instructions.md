# Boost Assembly Instructions

See [general assembly instructions](https://quinnfreedman.github.io/modular/docs/assembly)

## Components

**Most** parts are available on Tayda ([cart link](https://www.taydaelectronics.com/savecartpro/index/savenewquote/qid/14541852619), [quick-order CSV](https://freemodular.org/modules/Boost/fm_boost_tayda_bom.csv)).

See [general components notes](https://quinnfreedman.github.io/modular/docs/components) for more info about acquiring parts.

[Interactive BOM](https://quinnfreedman.github.io/fm-artifacts/Boost/boost_pcb_interactive_bom.html)

| Reference | Part             | Value                                   | Source  | Comment |
| --------- | ---------------- | --------------------------------------- | ------- | ------- |
| R1, R4    | Resistor         | 10kΩ                                    | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/10-x-resistor-10k-ohm-1-4w-1-metal-film-pkg-of-10.html) | R4 controls the current flowing through the clipping diodes. Using a different value would give a slightly different tone, especially if you want to use different diodes.  |
| R2        | Resistor         | 24kΩ                                    | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/10-x-resistor-24k-ohm-1-4w-1-metal-film-pkg-of-10.html) | Should be 1/2 the resistance of RV1. If you want more gain, you can make it smaller, but you may encounter op-amp clipping. |
| R3        | Resistor         | 2.4kΩ                                   | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/resistor-2-4k-ohm-1-4w-1-metal-film-pkg-of-10.html) | Should be 1/10x R2. |
| R5        | Resistor         | 2kΩ                                     | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/resistor-2k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
| R6        | Resistor         | 18kΩ                                    | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/resistor-18k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
| R7        | Resistor         | 750kΩ                                   | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/resistor-750k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
| R8        | Resistor         | 560kΩ                                   | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/resistor-560k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
| R9        | Resistor         | 15kΩ                                    | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/resistor-750k-ohm-1-4w-1-metal-film-pkg-of-10.html) | This resistor sets the cutoff frequency for the high-shelf filter which determines which frequencies are boosted/cut by the Tone knob. Feel free to adjust the value or swap it out for a trim pot to dial it in by ear. Just don't turn it all the way down to a short. |
| R10, R11  | Resistor         | 100kΩ                                   | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/10-x-resistor-100k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
| R12       | Resistor         | 1kΩ                                     | [Tayda](https://www.taydaelectronics.com/resistors/1-4w-metal-film-resistors/10-x-resistor-1k-ohm-1-4w-1-metal-film-pkg-of-10.html) | |
| RV1, RV2  | Potentiometer    | B50kΩ                                   | [Tayda](https://www.taydaelectronics.com/potentiometer-variable-resistors/rotary-potentiometer/50k-ohm-linear-taper-potentiometer-d-shaft-pcb-9mm.html) | Linear. You can use another value if you have it available. Match R2 and R3 to RV1. Any value for RV2 is probably fine with no changes but it might slightly change the response curve. Lower is probably better. |
| J1        | IDC connector    | 2x6                                     | [Tayda](https://www.taydaelectronics.com/10-pin-box-header-connector-2-54mm.html) | Eurorack power header. Can use two rows of male pin headers or a shrouded connector (recommended). |
| J2, J3    | 3.5mm Jack       | THONKICONN (a.k.a PJ398SM or PJ301M-12) | [Tayda](https://www.taydaelectronics.com/pj-3001f-3-5-mm-mono-phone-jack.html) | |
| D1, D2    | Diode            | 1N4148                                  | [Tayda](https://www.taydaelectronics.com/1n4148-switching-signal-diode.html) | You can use any diodes or LEDs here. Different diodes will have different response curves and will sound different. You may want to adjust R4 and/or the gain amount of the gain stages (see schematic) to get the diode in its non-linear operating range. |
| Q1        | Transistor       | 2N3904                                  | [Tayda](https://www.taydaelectronics.com/2n3904-npn-general-propose-transistor.html) | |
| C1        | Capacitor        | 470nF                                   | [Tayda](https://www.taydaelectronics.com/capacitors/ceramic-disc-capacitors/10-x-0-33uf-50v-ceramic-disc-capacitor-pkg-of-10.html) | This provides the AC coupling for the input stage. A 330nF is fine if you can't find a non-polarized 470nF cap in the right form factor. Or, you can replace it with a short circuit if you don't want AC coupling. There will still be a slight offset in voltage so the module won't be ideal for processing CV, but DC coupling will allow you to offset the audio signal for asymmetric clipping, which might sound more interesting. |
| C2        | Capacitor        | 10nF                                    | [Tayda](https://www.taydaelectronics.com/capacitors/ceramic-disc-capacitors/10-x-0-01uf-50v-ceramic-disc-capacitor-pkg-of-10.html) | |
| C3        | Capacitor        | 33pF                                    | [Tayda](https://www.taydaelectronics.com/capacitors/ceramic-disc-capacitors/10-x-33pf-50v-ceramic-disc-capacitor-pkg-of-10.html) | |
| C4, C5    | Capacitor        | 100nF                                   | [Tayda](https://www.taydaelectronics.com/capacitors/ceramic-disc-capacitors/a-553-0-1uf-50v-ceramic-disc-capacitor-pkg-of-10.html) | |
| C6-C7     | Capacitor        | 10uF                                    | [Tayda](https://www.taydaelectronics.com/10uf-16v-85c-radial-electrolytic-capacitor.html) | **Optional.** Power supply noise filtering capacitors |
| U1        | Op-amp           | TL074                                   | [Tayda](https://www.taydaelectronics.com/tl074-quad-operational-amplifier-j-fet-pdip-14-tl074cn.html) | |
