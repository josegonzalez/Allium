#!/bin/sh
cd /mnt/SDCARD/RetroArch/
HOME=/mnt/SDCARD/RetroArch/ ./retroarch -v -L .retroarch/cores/mgba_libretro.so "$1"