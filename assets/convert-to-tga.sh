#!/bin/bash
# convert an image into Bgr555 TGA format
# make sure image fits into 240x160 GBA display

[[ -z $1 ]] && { echo "usage: $0 image.jpg"; exit; }

out="${1%.*}.tga"
magick convert "$1" -set colorspace RGB -separate -swap 0,2 -combine -depth 5 "$out"

echo "wrote $out"
