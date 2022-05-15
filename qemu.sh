#!/bin/sh

# You might need to change this...
ESP_QEMU_PATH=~/src/qemu-espressif/build
BUILD=debug

TARGET=xtensa-esp32-espidf # Don't change this. Only the ESP32 chip is supported in QEMU for now

cargo espflash save-image --features qemu --merge target/$TARGET/$BUILD/rust-esp32-std-demo.bin
$ESP_QEMU_PATH/qemu-system-xtensa -nographic -machine esp32 -nic user,model=open_eth,id=lo0,hostfwd=tcp:127.0.0.1:7888-:80 -drive file=target/$TARGET/$BUILD/rust-esp32-std-demo.bin,if=mtd,format=raw
