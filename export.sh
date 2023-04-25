#!/bin/sh
PROJECT_DIR=$1
PROJECT_NAME=$2
OUTPUT_DIR="${2}_gerbers"
./kikit.sh fab jlcpcb --no-drc "$PROJECT_DIR/$PROJECT_NAME/$PROJECT_NAME.kicad_pcb" "$PROJECT_DIR/$OUTPUT_DIR"
mkdir -p "$PROJECT_DIR/$OUTPUT_DIR"
mv "$PROJECT_DIR/$OUTPUT_DIR/gerbers.zip" "$PROJECT_DIR/$OUTPUT_DIR.zip"
rm -r "$PROJECT_DIR/$OUTPUT_DIR"
