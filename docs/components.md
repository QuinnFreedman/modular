# Components

Each module will list the specific components required to build it in a **bill of materials**, which is in the assembly instructions for that module. To avoid repetition, common components that are used in many modules are listed here along with advice about where to buy them.

## Microcontrollers

The brain at the center of each digital module is a tiny computer called a microcontroller. I have used a [Arduino Nano v3](https://store.arduino.cc/usa/arduino-nano) in all my designs so far, although some future modules will probably require a more powerful controller.

You can buy first-party Arduino Nanos from the Arduino company for about $20 each, but you can also get cheep clones from third-party manufacturers for as little as $3, especially if you can buy a few at a time. Just check Amazon/Ebay/AliExpress for lots of 5 or 10.

Make sure the boards you buy have ATmega328 processors (the same processor on the genuine Nano v3) and operate at 16Mhz and 5 volts. Some cheap boards you find might use the weaker 168p processor or could be clocked at 8Mhz or 3.3 volts. In general, though, any board advertized as Nano v3 compatible should be equivalent and work fine. I have not personally found any 3rd party boards that didn't meet those criteria. The only other difference is that some third party boards may use a different USB chip which may require you to [install a driver](https://learn.sparkfun.com/tutorials/how-to-install-ch340-drivers/all) before you can flash them.

## Generic components

The exact model of these components doesn't matter. I recommend you buy them from Amazon, Ebay, or wherever is cheapest.

### Resistors

All my modules use 1/4 Watt THT resistors. These are the standard resistors that you will find in any DIY kit. The specific specs and tolerances do not matter. I recommend buying a kit of all different values together. This is probably cheaper and easier than buying the specific resistors listed in each module.

Smaller 1/8 Watt resistors would be fine too. None of my modules use nearly enough current to push any standard resistors near their tolerances.

### Capacitors

Like resistors, you might want to buy a capacitor kit with a range of values. My modules use some larger electrolytic capacitors as well as smaller ceramic capacitors. Again, the exact characteristics of the capacitors do not matter much here -- just make sure the capacitance value matches.

You can get them in bulk from [Mouser](https://www.mouser.com/ProductDetail/594-K104K10X7RF53H5) or in kits from [Amazon](https://www.amazon.com/BOJACK-Ceramic-Capacitor-Assortment-Capacitors/dp/B07P8N8BW9).

### LEDs

All my modules use 3mm lead-type/lamp-type LEDs. Again, you can buy a cheap kit with a variety of colors. All LEDs should be interchangeable, although some might be brighter than others, so you may need to adjust some resistor values to achieve the desired brightness. The specific resistors to adjust are noted in the bill of materials for each module.

Some modules also use 2-leg bipolar LEDs or 3-leg 2-color LEDs. These are probably easiest to buy from an electronics supplier like Mouse or DigiKey.

### Headers

Male/female pin headers allow you to connect stacked circuit boards to each other and to make removable sockets for Arduinos. Just buy a bunch and cut them down to size with wire cutters. E.g. from [here](https://www.amazon.com/gp/product/B01MQ48T2V).

2x8 male headers (or 2x5 in some cases) let you connect to the eurorack power supply. You could make these out of two rows of basic male pin headers but I highly recommend you just buy some shrouded headers so no one accidentally plugs in your module upside down and fries everything. I don't build in any reverse-current protection into my modules. You can get them e.g. form [here](https://www.amazon.com/uxcell-16-Pin-Straight-Shrouded-Connector/dp/B01N8XTFB5) or just search for "2x8 IDC male connectors".

## Potentiometers, Knobs, and Jacks

You can get most of these from [Thonk](https://www.thonk.co.uk/product-category/parts/).

### Pots

I use 15mm-shaft Alpha pots. You can get them [here](https://www.thonk.co.uk/shop/alpha-9mm-pots-dshaft/). Make sure you get a shaft type (D, spline, or smooth) that matches the knobs you want to use. The knobs I use are D-shaft only.

Also, pay attention to the difference between **linear** (B type) and **logarithmic** (A type) potentiometers. Most modules want linear pots, unless they are controlling audio levels. Confusingly, [some manufacturers use "A" and "B" to mean literally the opposite](https://en.wikipedia.org/wiki/Potentiometer#Resistance%E2%80%93position_relationship:_%22taper%22) of what they usually mean. But, Alpha pots, which I primarily use, seem to stick with "B" = linear.

If they don't have the potentiometer values you need in stock, most designs are very flexible. You should be able to use any value you have as long as you adjust some corresponding resistors to match. See notes in the module's bill of materials for more info.

### Knobs

I personally use [black Sifam/Selco skirted knobs (small and large)](https://www.thonk.co.uk/shop/intellijel-black-knobs/) for everything and design my modules with those in mind. For these knobs, you need to buy the [colored end caps](https://www.thonk.co.uk/shop/sifam-caps/) separately and press them in. I use the colors Red, Orange, Yellow, Green, Bright Aqua, Magenta, and white.

If you want to customize the look of your modules, just look at any DIY supply store like [Thonk](https://www.thonk.co.uk/product-category/parts/knobs/), [Modular Addict](https://modularaddict.com/parts/synth-diy-parts/knob), [Amplified Parts](https://www.amplifiedparts.com/products/knobs), or [Love My Switches](https://lovemyswitches.com/knobs/). Or, get cheap lots on amazon or ebay (like [this](https://www.amazon.com/gp/product/B073BCR8T6), or [this](https://www.amazon.com/gp/product/B073BCR8T6)) Any knobs should be compatible as long as you match the shaft type to the potentiometers you are using.

### Jacks

I use the THONKICONN style jacks from Thonk (a.k.a PJ301M-12 or PJ398SM).[Buy them here](https://reverb.com/item/16036916-thonk-50-pack-3-5mm-jack-sockets-thonkiconn-with-knurled-nuts) or [here](https://www.thonk.co.uk/shop/thonkiconn/).

### Rotary encoders

A few of my modules use rotary encoders in addition to potentiometers. I use EC11M D-type encoders. I have had trouble finding them from suppliers, but you can get them on amazon [here](https://www.amazon.com/DIYhz-Rotary-Encoder-Digital-Potentiometer/dp/B07D3DF8TK/).

### Switches

When I use switches in my projects, I use the Taiway Sub-Mini SPDT's -- both the ON/ON's (two position) and the ON/OFF/ON's (three position). You can get them from [Love My Switches](https://lovemyswitches.com/taiway-sub-mini-spdt-on-on-switch-pcb-mount-long-shaft/), [Thonk](https://www.thonk.co.uk/shop/sub-mini-toggle-switches/), or [Amplified Parts](https://www.amplifiedparts.com/products/switch-carling-submini-toggle-spdt-2-position-pc-pins). Make sure you get the "PCB mount" version so they can solder directly into the PCB. If you want a different size or style of switch, just adjust the faceplate cutout to match.

## IC chips

All other components (integrated circuit chips, transistors, diodes, etc.) should have a specific part number specified in the module's bill of materials. You can get them from a supplier like [DigiKey](https://digikey.com/) or [Mouser](https://mouser.com).

In some cases there will be multiple skews of a single chip, specified by additional letters after the part number. For example, if you look up the TL072 op-amp on DigiKey, you will see TL072CP, TL072IP, TL072BPC, etc. Usually, these just indicate different grades (different heat tolerance, precision, etc). In general, you can just get whatever is cheap and available. But, look out for different packages. My modules use through-hole (aka large, breadboard-sized) parts wherever possible because they are easier to solder, but most chips also come in various smaller versions. Filter for `Mounting Type: Through Hole/THT` or `Package: DIP` when searching for parts.
