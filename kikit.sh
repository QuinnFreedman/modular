#!/bin/sh
/usr/bin/flatpak run --branch=stable --arch=x86_64 --command="python3" --file-forwarding org.kicad.KiCad @@ -c "from kikit.ui import cli; cli()" $@ @@

