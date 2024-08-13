# Assembly

The assembly process is very similar for most modules, so I will describe the general procedure here. Specific information for each module can found in the documentation for that module. Basically, you'll need to:

1. **Download** the GERBER files from each module's page
2. **Upload** the files to a PCB manufacturer like JLCPCB
3. **Buy** the parts listed in the module's **bill of materials**
4. **Solder** everything together
5. **Flash** the firmware onto the Arduino by USB (if applicable)

## What you will need

For information about the components you will need and where to get them, see the [components](https://quinnfreedman.github.io/modular/docs/components) page. Module-specific components may also be listed in the module's documentation.

In addition to the electrical components required for the module, you will need a soldering iron and some solder. It may also be useful to have a multimeter to check your work and a solder wick to fix your mistakes.

## Soldering

All the components of the module will be held together with solder. If you have never soldered before, don't worry! It's very easy and you will learn quickly. There are numerous videos on YouTube showing you how to do it. At first, your solder joints might look a little ugly but they should still work fine and it is very unlikely that you will damage any components in the process.

Make sure that your space is well ventilated (or use a fume extractor) and make sure to wash your hands and table afterward if you are using led solder.

## PCB

I highly recommend using printed circuit boards to make these modules. Each module has links to GERBER files which describe the PCB layout. Upload these to a manufacturing service like [PCBWay](https://www.pcbway.com/) or [JLCPCB](https://jlcpcb.com/order) to get them printed. 

Most modules are built out of a sandwich of two PCBs, so you will need to print them both.

When filling out an order, you can leave all the settings default. These PCBs do not require any special features or materials. I also try to keep all my PCBs under 100mmx100mm, which means you should get a super cheap flat rate with most manufacturers.

Some manufacturers require files in slightly different formats. GERBERs for JLCPCB and PCBWay are pre-generated for each board. If you want to use another service, try the PCBWay files. If that doesn't work, open the source files in KiCad and re-export the files you need following your manufacturers instructions.

If you want to make the modules on stripboard, a schematic PDF for each module should be linked in the module's documentation, but you will need to do the stripboard layout yourself.

## Faceplate

There is an SVG image file linked in each module's documentation with my design for the faceplate. You can print this out and use it as a stencil for marking and drilling a sheet of aluminum. If you print a stencil, make sure that it is 128.5mm tall. If not, that is an indication that the scale was messed up in the printing.

Alternatively, you can use the same service that you used for the PCBs to print a faceplate. Each module also has GERBER files for the faceplate, which can be printed as if it was a PCB. JLCPCB also offers aluminum-backed PCBs, which I highly recommend for this.

## Assembly

Each module should have a file with assembly instructions and materials. Read the comments for module-specific instructions. Additionally, each PCB has an interactive BOM, linked in the module docs, which helps you keep track of which components you need to place and where they go.

Other than that, there is no special trick to this. Put each component into the PCB in the location indicated by the silkscreen, and then solder them in place. It's general recommend to start will the smallest pieces (like resistors) and work up to the tallest components (like potentiometers). This makes it easier to lay the board flat on its back for soldering. Before soldering any panel components (like jacks, potentiometers, and switches) I recommend assembling the whole module and screwing them firmly onto the faceplate in order to make sure everything is properly aligned.

I use single row pin headers to attach boards to each other in a way that is removable. When soldering these, it is best to assemble both sides together before soldering so that you can make sure they line up. This will also help hold the headers in place. Similarly, when you solder on the headers that will hold the Arduino, it is easiest to put the Arduino in them and then solder them on as one unit.

## Checking for errors

If you want to check your module for errors before plugging it in for the first time, you can use a multimeter. Set the meter to continuity or resistance mode and touch the probes to each combination of the power pins (ground, +5v, -12v and +12v). None of these pins should be shorted to each other. Also, none of the audio/CV inputs/outputs should be shorted directly to ground (or power) when a cable is plugged in (although some of them may be connected to ground through a resistor, and some may be "normalled" to ground when a cable is disconnected). Plug in a cable to each jack and check for shorts between the shank of the cable (aka the collar or ring) and the tip, or between the tip and the power pins.

## Firmware

If the project you are assembling involves an Arduino, then you will need to load the right program onto that Arduino in order for the module to work.

Each module's documentation should link to a HEX file. That file is the program that needs to be uploaded to the arduino. There are multiple ways to do that, but none of them are especially user friendly. 

At some point in the future, I will probably make a dedicated tool for flashing firmware. Until then, I recommend using [avrdudess](https://github.com/ZakKemble/AVRDUDESS). If you are on windows, you can download the latest release from the [releases page](https://github.com/ZakKemble/AVRDUDESS/releases) and it should just work. If you are on Mac or Linux, refer to the documentation.

Select "Arduino Nano (ATmega328P)" from the presets dropdown. Select the HEX file you downloaded in the in the "Flash" section. To pick the right Port, check the ports list, then plug in your Arduino to a USB port and see if any new ports appear. On Windows, it should be something like `COM1`. On Linux, it will probably be `/dev/ttyUSB0`. Just try all the available ports in ascending order until something works. If you think you have the right port but it still isn't working, you could also try reducing the Baud rate to something like 9600.

If you are comfortable with the command line, you can also use [avrdude](https://github.com/avrdudes/avrdude) or [ravedude](https://github.com/Rahix/avr-hal/blob/main/ravedude/README.md). Or, just google "How to flash hex file to Arduino" for your operating system and seeing if any of the results are helpful.
