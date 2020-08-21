This repository is a collection of open source hardware & software designs for **Eurorack synthesizer modules**, designed to be **cheap** and **easy** to assemble at home, primarily based on **Arduino microcontrollers**.

So what does that mean?

## What is Modular Synthesis (and Eurorack)?

A *synthesizer* is a physical tool used to create **analog electrical signals** which will be interpreted as audio by a speaker or recording device. The methods used to create these signals may include analog and/or digital components. You are probably used to seeing synthesizers that look a little bit like pianos, with a row of keys mapped to different notes, but this does not have to be the case.

A *modular* synthesizer is a kind of synthesizer made up of modules. Each module performs a specific function (like creating a sound, filtering a sound, sequencing notes, etc.) and modules can be connected together with wires called patch cables.

At its most basic level, this lets you make your ideal traditional synthesizer -- combining the oscillators, filters, and effects that you like with your favorite keyboard. But, the real magic of modular synthesis comes from the idea of **modulation**. Modules don't just input and output audio signals, but also control signals called **control voltage**. Every knob or button on a module -- like the cutoff of a filter or the attack of an envelope -- can be controlled by other modules in the system, using the same patch cables that carry audio signals. This gives you the power to create any kind of instrument you can imagine just by patching modules together in different configurations.

Your modular synth can be a traditional keyboard-driven voice, or a generative instrument that creates its own non-repeating melodies. It can be a dedicated effects unit that just processes external sound, or a drum machine that plays back samples according to a sequencer. And, it can be all these things by just re-arranging some cables.

*Eurorack* is just a standard for modules. All these modules need to fit into a the same case, get their power from the same power supply, and output the same range of voltages so they don't fry eachother. There are many different form factors for modular synthesizers, but Eurorack is by far the most popular. Eurorack modules are about 5 inches tall, work off of 12 volt power, and output +/-10v signals. All the modules in this repository are designed for Eurorack but it shouldn't be too hard to adapt them to the standard of your choice.

Here are some videos that explain modular better than I can:

[![What is a modular synth?](https://img.youtube.com/vi/ex36kK8YpQo/0.jpg)](https://www.youtube.com/watch?v=ex36kK8YpQo)

[![What is a modular synth?](https://img.youtube.com/vi/oBEZF2pAbMg/0.jpg)](https://www.youtube.com/watch?v=oBEZF2pAbMg)


## What is the purpose of this project?

Modular synthesis is a profoundly democratic music. It makes musical exploration accessible to a wide range of people who are not interested in learning a traditional instrument or who find it natural to think about music in different ways. Unfortunately, Eurorack hardware is also a prohibitively expensive. Individual modules usually cost anywhere from $150 to $500 and you need a bare minimum of around 4-6 modules to make any kind of meaningful sound, with many enthusiasts collecting hundreds or more, not to mention all the associated cases, cables, power supplies, amps, and audio interfaces. This means that for many people, including myself, getting into hardware modular synthesis is either impossible or at least a dubious financial decision.

But, it doesn't have to be that way. Module manufacturers are justified in charging high prices -- many modules are hand-assembled from numerous analog components, not to mention the countess hours of R&D involved in designing each new module (I would know). But, the rise of ultra cheep, widely available digital electronics means that, with a little bit of careful design, these modules can be made for a fraction of the cost.

**My goal is for every module in this repo to be able to be built for $20-$30 and about an hour of work with little technical skill, making the world of modular synthesis available to a much wider range of people.** (This cost is amortized so you might have to pony up $100+ for your first module for components).

This project is secondarily intended **to provide a resource for people who want to get into designing their own modules from scratch**, especially using digital components. All the software and hardware is, of course, open source, and is intended to be readable and comprehensable. As time goes on, I will add more documentation about the decisions I made when designing the modules so that others can more easily modify them or make their own.

## What do I need?

If you already have a Eurorack synth, you can get right to building. If not, you will need to buy or make a case, power supply, and audio output module. See the [peripherals](peripherals.md) page for more details. You will probably also want to buy some commercial modules -- at least in the beginning -- since this project won't have amassed enough module designs to make a full instrument for a little while. In the meantime, you can use the mony you save from building your own modules to justify buying the really high-end filters and effects you want.

## Where do I start?

Each folder in [this GitHub repo](https://github.com/QuinnFreedman/modular) represents a different module. Read the `README` file inside each one for more info about the module.

Once you have picked a module you want to make, follow the generic [assembly instructions](assembly.md). Refer back to the module's `README` file for specific instructions for that module.


## Other resources

There are surprisingly few resources about making your own Eurorack modules from scratch (as opposed to buying a DIY kit for a manufacturer) and even fewer of these involve Arduinos or other microprocessors.

A few things that I did find useful when working on these modules:

* **[Look Mum No Computer](https://www.youtube.com/lookmumnocomputer)**. Sam is a musician and creator who makes almost all his own modules for his synthesizer. Many of the schematics for these modules are available on [his website](https://www.lookmumnocomputer.com/projects) and others are available for purchase. His modules are **not** Eurorack (they are his own format which he calls Kosmo), but they are close enough that you could probably adapt them.
* **[Mutable Instruments](https://mutable-instruments.net)** is a commercial Eurorack module manufacturer, but Émilie Gillet makes all the code and hardware for their modules open source. These sources are not necesarily intended to be readable, but the modules are super high quality and having any open source refferencecs to go on is a huge help.
* **[Music Thing Modular](https://musicthing.co.uk/)** is another module manufacturer that open-sources all their modules. It's hard to understate how helpful it its to see the way real commercial modules are built to give yourself confidence that what you are doing is ok. My RNG module is also directly inspired by the MTM Turing Machine (in function, at least -- not in design).
* **[Yusynth](https://yusynth.net/index_en.php)** seems to have a lot of amazing designs for oldschool 5U modules. The website is largely inscrutable to me, but some day I plan to try to decipher it and maybe translate some of the modules into Eurorack.
* **[Otto's DIY](https://ottosdiy.com/)** makes the power adapters I use. They also have a few Arduino-based (and non-arduino) "recipes" although I haven't actually looked at them.
* **[DU-INO](https://www.youtube.com/watch?v=4XBag8quOJ8)** is a Detroit Underground module that can be controlled by an arduino. I haven't played with one but for someone who wants to start making modules with an arduino without any soldering, it might be a good place to start.

# License

The software in this repo is [GPL3.0](https://www.gnu.org/licenses/gpl-3.0.html)

Everything else is [cc-by-sa 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
