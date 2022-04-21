#!/bin/sh

# You might need to change this...
ESP_QEMU_PATH=../qemu/build
BUILD=debug

TARGET=xtensa-esp32-espidf # Don't change this. Only the ESP32 chip is supported in QEMU for now

esptool.py --chip esp32 elf2image target/$TARGET/$BUILD/rust-esp32-std-hello
esptool.py --chip esp32 merge_bin --output target/$TARGET/$BUILD/rust-esp32-std-hello-qemu.bin --fill-flash-size 4MB 0x1000 qemu_bins/bootloader.bin  0x8000 qemu_bins/partitions.bin  0x10000 target/$TARGET/$BUILD/rust-esp32-std-hello.bin --flash_mode dio --flash_freq 40m --flash_size 4MB
$ESP_QEMU_PATH/qemu-system-xtensa -nographic -machine esp32 -nic user,model=open_eth,id=lo0,hostfwd=tcp:127.0.0.1:7888-:80 -drive file=target/$TARGET/$BUILD/rust-esp32-std-hello-qemu.bin,if=mtd,format=raw
