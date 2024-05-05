In addition to the modules in this repository, you will need a few extra components to make a working synth. If you are new to Eurorack, you will need a case, power supply, and cables. Additionally, there aren't enough modules in this project to make a fully working synth **(yet)** so you will probably want to supplement with a few commercial modules.

## Cases

Modules on their own are just a faceplate with some circuit boards on the back. To make them usable, you will probably want a case to mount them in.

Cases are one of the most common parts to DIY. A case can really be anything as long as it has mounting rails the right distance apart. You should be able to find a bunch of examples of cool, home-built cases online. If you want to build a case yourself, you can find everything you need (rails, threaded inserts, brackets, screws, etc.) from [Modular Synth Lab](https://modularsynthlab.com/product-category/diy-eurorack-case/).

If you want to buy a case, here are some ideas:

1. [TipTop Happy Ending](http://tiptopaudio.com/happyendingkit/) is pretty much the cheapest "case" in the game. It's what I personally used when I first started out. For the price (80% of which is just the power supply) you can't really complain, but it's not the nicest looking option. It also doesn't come with any bonus features like a built-in midi input or audio output that some other small cases have.
2. [TipTop Mantis](http://tiptopaudio.com/mantis) may be the best HP-per-dollar value out there. But, again, I'm not a huge fan of the plastic-y look of this case and the lack of features.
3. [Intellijel's cases](https://intellijel.com/shop/cases/) are a little pricier but very popular. They have some I/O, power, and other cool features built into the case itself, plus you get to use Intellijel's 1U tiles.
4. The [Nifty Case](https://www.cre8audio.com/niftycase) is another option that is popular with beginners. Again, there is some stuff built in to the case (power, audio out and midi in). I haven't used it personally, but is probably what I would recommend for someone starting out. When you take into account the price of a power supply, audio interface and midi input, it's the best deal I have found. One thing to keep in mind: all my designs are skiff-friendly, meaning they will fit in this case, but most other DIY projects mount their PCBs sideways and so might not fit in this case.
5. The [Befaco Jumpskiff](https://shop.befaco.org/en/power-solutions/506-jumpskiff-diy-kit.html) is a pretty cool-looking DIY kit.
6. [4MS Pods](https://4mscompany.com/pods.php) are super tiny cases that can be chained together as your collection grows.
7. [Ikocase](https://www.etsy.com/shop/Ikocase) is a hand manufacturer of wooden cases with built-in power supplies on Etsy at pretty competitive costs.

## Power Supplies

Modules need power. Some cases come with power supplies builtin while others require you to buy dedicated power modules. 
I personally use [Synthrotek's Case Power (Green)](https://www.synthrotek.com/products/modular-circuits/case-power/) for my larger case and this much cheaper [Frequency Central DIY kit](https://frequencycentral.co.uk/product/fc-power/) for my test bench. Both work excellently for me, but do a little research to find a power supply that meets your needs. Here is a much more in-depth guide from [Perfect Circuit](https://www.perfectcircuit.com/signal/eurorack-modular-power-basics).

I don't give power consumption ratings for my modules. Maybe I will figure that out at some point. None of them should draw unusually much, though. I recommend getting a power supply that is suited to the approximate size of case you want to build plus a little head room.

## Power Cables

You will need to plug your modules into your power supply using ribbon cables like [this](https://www.sweetwater.com/store/detail/4msMultPwr16P--4ms-multi-power-cable-16-pin). They will all be 16 pins (2x8) on the side that plugs into the power supply, but they may be 16 or 10 pins ([like this](https://www.sweetwater.com/store/detail/EuroPC10-16--tiptop-audio-10-to-16-pin-eurorack-module-power-cable)) on the module side depending on if the module requires 5v power or not. You can buy them in bulk for not too much money but you can also very easily make your own. You can buy spools of ribbon cable (probably 20 wires wide), cut it into lengths, strip it down to 10 or 16 wide, and then crimp some "IDC" connectors on the end. You can crimp them with a hammer or pair of pliers but a $15 "IDC crimp tool" makes the process much easier.

**Note: Make sure you orient your cables correctly! For Eurorack cables, the red wire (carrying -12v) always faces DOWN, and the bump on the connector always faces RIGHT when you plug it in (no matter which side you are plugging in).**

[Here is a very detailed video](https://www.youtube.com/watch?v=tok3l28D55k) outlining the whole process of making or salvaging ribbon cables.

## Patch Cables

You will need a large collection of 3.5mm mono audio cables to patch your modules together. You can get a bundle from pretty much any synth seller and they are all of similar quality. The [Stackcables](https://tiptopaudio.com/stackcable/) from TipTop Audio are especially nice, but a bit pricey.

If you have some spare "Aux" cables lying around, you might be able to use those as well. Standard audio cables are stereo rather than mono, so they might not fit correctly in a Eurorack socket. But, if you find some that work for you that is probably the cheapest option.
