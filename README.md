# Rust on ESP32 STD demo app

A demo STD binary crate for the ESP32[XX] and ESP-IDF, which connects to WiFi, Ethernet, drives a small HTTP server and draws on a LED screen.

![CI](https://github.com/ivmarkov/rust-esp32-std-demo/actions/workflows/ci.yml/badge.svg)

[Join in](https://matrix.to/#/#esp-rs:matrix.org) on the discussion!

Highlights:

- **Pure Rust and pure Cargo build!** No CMake, no PlatformIO, no C helpers
  - ... via [esp-idf-sys](https://crates.io/crates/esp-idf-sys) and [embuild](https://crates.io/crates/embuild)
- **Support for Rust STD** (threads, console, TCP/IP) safe APIs
  - ... upstreamed and [part of the Rust STD library](https://github.com/rust-lang/rust/pull/87666)
- **New, experimental!** Support for asynchronous networking using [smol](https://github.com/smol-rs/smol)
- Support for running in the [Espressif fork of QEMU](https://github.com/espressif/qemu/wiki)
- Rust Safe APIs for various ESP-IDF services like WiFi, Ping, Httpd and logging
  - ... via [esp-idf-svc](https://crates.io/crates/esp-idf-svc) ([embedded-svc](https://crates.io/crates/embedded-svc) abstractions implemented on top of ESP-IDF)
- NAPT support (Router from the SoftAP to the STA interface). **NOTE**: In production, do NOT leave the SoftAP interface open (without password)!
- Driving a LED screen with the [embedded-graphics](https://crates.io/crates/embedded-graphics) Rust crate
  - ... via [esp-idf-hal](https://crates.io/crates/esp-idf-hal) ([embedded-hal](https://crates.io/crates/embedded-hal) drivers implemented on top of ESP-IDF)
- (ESP32-S2 only) [Blink a LED](https://github.com/ivmarkov/rust-esp32-ulp-blink) by loading a pure Rust program onto the RiscV Ultra Low Power CPU

## Build

- Install the [Rust Espressif compiler toolchain and the Espressif LLVM Clang toolchain](https://github.com/esp-rs/rust-build)
  - This is necessary, because support for the Xtensa architecture (ESP32 / ESP32-S2 / ESP32-S3) is not upstreamed in LLVM yet
- Switch to the `esp` toolchain from the pre-built binaries: `rustup default esp`
  - (You can also skip this step and switch to the `esp` toolchain *for the demo crate only* by executing `rustup override set esp` inside the `rust-esp32-std-demo` directory once you have cloned the demo as per below)
  - **NOTE** For ESP32-C3 - which runs a RiscV32 chip - you can just use the stock nightly Rust compiler, and a recent, stock Clang (as in Clang 11+)
  - (You can do this by issuing `rustup install nightly` and then `rustup default nightly` instead of installing/building the Rust & Clang ESP forks and switching to their `esp` toolchain as advised above)
- If using the custom Espressif Clang, make sure that you DON'T have a system Clang installed as well, because even if you have the Espressif one first on your `$PATH`, Bindgen will still pick the system one
  - A workaround that does not require uninstalling the system Clang is to do `export LIBCLANG_PATH=<path to the Espressif Clang lib directory>` prior to continuing the build process
- `cargo install ldproxy`
- Clone this repo: `git clone https://github.com/ivmarkov/rust-esp32-std-demo`
- Enter it: `cd rust-esp32-std-demo`
- Export two environment variables that would contain the SSID & password of your wireless network:
  - `export RUST_ESP32_STD_DEMO_WIFI_SSID=<ssid>`
  - `export RUST_ESP32_STD_DEMO_WIFI_PASS=<ssid>`
- To configure the demo for your particular board, please uncomment the relevant [Rust target for your board](https://github.com/ivmarkov/rust-esp32-std-demo/blob/main/.cargo/config.toml#L2) and comment the others. Alternatively, just append the `--target <target>` flag to all `cargo build` lines below.
- Build: `cargo build` or `cargo build --release`
  - (Only if you happen to have a [TTGO T-Display board](http://www.lilygo.cn/prod_view.aspx?TypeId=50033&Id=1126&FId=t3:50033:3)): Add `ttgo` to the `--features` build flags above (as in `cargo build --features ttgo`) to be greeted with a `Hello Rust!` message on the board's LED screen
  - (Only if you happen to have a [Waveshare board](https://www.waveshare.com/wiki/E-Paper_ESP32_Driver_Board) and a [waveshare 4.2" e-paper screen](https://www.waveshare.com/wiki/4.2inch_e-Paper_Module)): Add `waveshare_epd` to the `--features` build flags above (as in `cargo build --features waveshare_epd`) to be greeted with a `Hello Rust!` message on the e-paper screen
  - (Only if you happen to have an [ESP32-S2-Kaluga-1 board](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s2/hw-reference/esp32s2/user-guide-esp32-s2-kaluga-1-kit.html)): Add `kaluga` to the `--features` build flags above (as in `cargo build --features kaluga`) to be greeted with a `Hello Rust!` message on the board's LED screen
  - (Only if you happen to have a [Heltec LoRa 32 board](https://heltec.org/project/wifi-lora-32/)): Add `heltec` to the `--features` build flags above (as in `cargo build --features heltec`) to be greeted with a `Hello Rust!` message on the board's LED screen
  - (Only if you happen to have an [ESP32-S3-USB-OTG](https://www.espressif.com/en/products/devkits)): Add `esp32s3_usb_otg` to the `--features` build flags above (as in `cargo build --features esp32s3_usb_otg`) to be greeted with a `Hello Rust!` message on the board's LED screen
  - (Only if you happen to have an [Ethernet-to-SPI board based on the W5500 chip](https://www.wiznet.io/product-item/w5500/)): Add `w5500` to the `--features` build flags above (as in `cargo build --features w5500`) to have Ethernet connectivity as part of the demo
    - Note that other Ethernet-to-SPI boards might work just fine as well, but you'll have to change the chip from `SpiEthDriver::W5500` to whatever chip your SPI board is using, in the demo code itself.
  - (Only if you happen to have an [ESP32 board with an onboard IP101 LAN chip and/or a stock ESP32 board connected to an IP101 Ethernet board via RMII](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/hw-reference/esp32/get-started-ethernet-kit.html)): Add `ip101` to the `--features` build flags above (as in `cargo build --features ip101`) to have Ethernet connectivity as part of the demo
    - Note that other RMII Ethernet boards might work just fine as well, but you'll have to change the chip from `RmiiEthDriver::IP101` to whatever chip your board is using, in the demo code itself.
- (Only if you happen to have an ESP32-S2 board and can connect a LED to GPIO Pin 04 and GND): Try accessing `http://<dhcp-ip-of-the-board>>/ulp` once build is flashed on the MCU

## QEMU

- Rather than flashing on the chip, you can now run the demo in QEMU:
  - Clone and then build [the Espressif fork of QEMU](https://github.com/espressif/qemu) by following the [build instructions](https://github.com/espressif/qemu/wiki)
  - Uncomment `CONFIG_ETH_USE_OPENETH=y`, `CONFIG_MBEDTLS_HARDWARE_AES=n`, and `CONFIG_MBEDTLS_HARDWARE_SHA=n` in `sdkconfig.defaults.esp32` (it is not enabled by default because this somehow causes issues when compiling for the ESP32S2)
  - Build the app with `cargo build --features qemu`
  - NOTE: Only ESP32 is supported for the moment, so make sure that the `xtensa-esp32-espidf` target (the default one) is active in your `.cargo/config.toml` file (or override with `cargo build --features qemu --target xtensa-esp32-espidf`)
  - Run it in QEMU by typing `./qemu.sh`. NOTE: You might have to change the `ESP_QEMU_PATH` in that script to point to the `build` subdirectory of your QEMU Espressif clone

## Flash

- `cargo install espflash`
- `espflash /dev/ttyUSB0 target/[xtensa-esp32-espidf|xtensa-esp32s2-espidf|riscv32imc-esp-espidf]/debug/rust-esp32-std-demo`
- Replace `dev/ttyUSB0` above with the USB port where you've connected the board

**NOTE**: The above commands do use [`espflash`](https://crates.io/crates/espflash) and NOT [`cargo espflash`](https://crates.io/crates/cargo-espflash), even though both can be installed via Cargo. `cargo espflash` is essentially `espflash` but it has some extra superpowers, like the capability to build the project before flashing, or to generate an ESP32 .BIN file from the built .ELF image.

## Alternative flashing

- You can also flash with the [esptool.py](https://github.com/espressif/esptool) utility which is part of the Espressif toolset
- Use the instructions below **only** if you have flashed successfully with `espflash` at least once, or else you might not have a valid bootloader and partition table!
- The instructions below only (re)flash the application image, as the (one and only) factory image starting from 0x10000 in the partition table!
- Install esptool using Python: `pip install esptool`
- (After each cargo build) Convert the elf image to binary: `esptool.py --chip [esp32|esp32s2|esp32c3] elf2image target/xtensa-esp32-espidf/debug/rust-esp32-std-demo`
- (After each cargo build) Flash the resulting binary: `esptool.py --chip [esp32|esp32s2|esp32c3] -p /dev/ttyUSB0 -b 460800 --before=default_reset --after=hard_reset write_flash --flash_mode dio --flash_freq 40m --flash_size 4MB 0x10000 target/xtensa-esp32-espidf/debug/rust-esp32-std-demo.bin`

## Monitor

- Once flashed, the board can be connected with any suitable serial monitor, e.g.:
  - ESPMonitor: `espmonitor /dev/ttyUSB0` (you need to `cargo install espmonitor` first)
  - Cargo PIO (this one **decodes stack traces**!): `cargo pio espidf monitor /dev/ttyUSB0` (you need to `cargo install cargo-pio` first)
    - Please run it from within the `rust-esp32-std-demo` project directory, or else the built ELF file will not be detected, and the stack traces will not be decoded!
  - Built-in Linux/MacOS screen: `screen /dev/ttyUSB0 115200` (use `Ctrl+A` and then type `:quit` to stop it)
  - Miniterm: `miniterm --raw /dev/ttyUSB0 115200`

- If the app starts successfully, it should be listening on the printed IP address from the WiFi connection logs, port 80.

- Open a browser, and navigate to one of these:
  - `http://<printed-ip-address>`
  - `http://<printed-ip-address>/foo?key=value`
  - `http://<printed-ip-address>/bar`
  - `http://<printed-ip-address>/ulp` (ESP32-S2 only)

- The monitor should output more or less the following:
```
Hello, world from Rust!
More complex print [foo, bar]
Rust main thread: ...
This is thread number 0 ...
This is thread number 1 ...
This is thread number 2 ...
This is thread number 3 ...
This is thread number 4 ...
About to join the threads. If ESP-IDF was patched successfully, joining will NOT crash
Joins were successful.
I (4761) wifi:wifi driver task: 3ffc1d80, prio:23, stack:6656, core=0
I (4761) system_api: Base MAC address is not set, read default base MAC address from BLK0 of EFUSE
I (4761) system_api: Base MAC address is not set, read default base MAC address from BLK0 of EFUSE
I (4771) wifi:wifi firmware version: 3ea4c76
I (4771) wifi:config NVS flash: disabled
I (4781) wifi:config nano formating: disabled
I (4781) wifi:Init dynamic tx buffer num: 32
I (4791) wifi:Init data frame dynamic rx buffer num: 32
I (4791) wifi:Init management frame dynamic rx buffer num: 32
I (4801) wifi:Init management short buffer num: 32
I (4801) wifi:Init static rx buffer size: 1600
I (4811) wifi:Init static rx buffer num: 10
I (4811) wifi:Init dynamic rx buffer num: 32
I (4811) esp_idf_svc::wifi: Driver initialized
I (4821) esp_idf_svc::wifi: Event handlers registered
I (4821) esp_idf_svc::wifi: Initialization complete
I (4831) rust_esp32_std_demo: Wifi created
I (4831) esp_idf_svc::wifi: Setting configuration: Client(ClientConfiguration { ssid: "<your-ssid>", bssid: None, auth_method: WPA2Personal, password: "<your-pass>", ip_conf: Some(DHCP) })
I (4851) esp_idf_svc::wifi: Stopping
I (4861) esp_idf_svc::wifi: Disconnect requested
I (4861) esp_idf_svc::wifi: Stop requested
I (4871) esp_idf_svc::wifi: About to wait for status
I (4871) esp_idf_svc::wifi: Providing status: Status(Stopped, Stopped)
I (4881) esp_idf_svc::wifi: Waiting for status done - success
I (4881) esp_idf_svc::wifi: Stopped
I (4891) esp_idf_svc::wifi: Wifi mode STA set
I (4891) esp_idf_svc::wifi: Setting STA configuration: ClientConfiguration { ssid: "<your-ssid>", bssid: None, auth_method: WPA2Personal, password: "<your-pass>", ip_conf: Some(DHCP) }
I (4911) esp_idf_svc::wifi: Setting STA IP configuration: DHCP
I (4921) esp_idf_svc::wifi: STA netif allocated: 0x3ffc685c
I (4921) esp_idf_svc::wifi: STA IP configuration done
I (4931) esp_idf_svc::wifi: STA configuration done
I (4931) esp_idf_svc::wifi: Starting with status: Status(Starting, Stopped)
I (4941) esp_idf_svc::wifi: Status is of operating type, starting
I (5041) phy: phy_version: 4180, cb3948e, Sep 12 2019, 16:39:13, 0, 0
I (5041) wifi:mode : sta (f0:08:d1:77:68:f0)
I (5041) esp_idf_svc::wifi: Got wifi event: 2
I (5051) esp_idf_svc::wifi: Recconecting
I (5051) esp_idf_svc::wifi: Start requested
I (5051) esp_idf_svc::wifi: Set status: Status(Started(Connecting), Stopped)
I (5061) esp_idf_svc::wifi: About to wait for status with timeout 10s
I (5071) esp_idf_svc::wifi: Wifi event 2 handled
I (5091) esp_idf_svc::wifi: Providing status: Status(Started(Connecting), Stopped)
I (5171) wifi:new:<1,1>, old:<1,0>, ap:<255,255>, sta:<1,1>, prof:1
I (5941) wifi:state: init -> auth (b0)
I (5951) esp_idf_svc::wifi: Providing status: Status(Started(Connecting), Stopped)
I (5951) wifi:state: auth -> assoc (0)
I (5961) wifi:state: assoc -> run (10)
I (5981) wifi:connected with muci, aid = 1, channel 1, 40U, bssid = 08:55:31:2e:c3:cf
I (5981) wifi:security: WPA2-PSK, phy: bgn, rssi: -54
I (5981) wifi:pm start, type: 1

I (5991) esp_idf_svc::wifi: Got wifi event: 4
I (5991) esp_idf_svc::wifi: Set status: Status(Started(Connected(Waiting)), Stopped)
I (6001) esp_idf_svc::wifi: Wifi event 4 handled
I (6011) wifi:AP's beacon interval = 102400 us, DTIM period = 1
I (6451) esp_idf_svc::wifi: Providing status: Status(Started(Connected(Waiting)), Stopped)
I (6951) esp_idf_svc::wifi: Providing status: Status(Started(Connected(Waiting)), Stopped)
I (7451) esp_idf_svc::wifi: Providing status: Status(Started(Connected(Waiting)), Stopped)
I (7951) esp_idf_svc::wifi: Providing status: Status(Started(Connected(Waiting)), Stopped)
I (8221) esp_idf_svc::wifi: Got IP event: 0
I (8221) esp_idf_svc::wifi: Set status: Status(Started(Connected(Done(ClientSettings { ip: 192.168.10.155, subnet: Subnet { gateway: 192.168.10.1, mask: Mask(24) }, dns: None, secondary_dns: None }))), Stopped)
I (8231) esp_idf_svc::wifi: IP event 0 handled
I (8241) esp_netif_handlers: staSTA netif allocated:  ip: 192.168.10.155, mask: 255.255.255.0, gw: 192.168.10.1
I (8451) esp_idf_svc::wifi: Providing status: Status(Started(Connected(Done(ClientSettings { ip: 192.168.10.155, subnet: Subnet { gateway: 192.168.10.1, mask: Mask(24) }, dns: None, secondary_dns: None }))), Stopped)
I (8461) esp_idf_svc::wifi: Waiting for status done - success
I (8461) esp_idf_svc::wifi: Started
I (8471) esp_idf_svc::wifi: Configuration set
I (8471) rust_esp32_std_demo: Wifi configuration set, about to get status
I (8481) esp_idf_svc::wifi: Providing status: Status(Started(Connected(Done(ClientSettings { ip: 192.168.10.155, subnet: Subnet { gateway: 192.168.10.1, mask: Mask(24) }, dns: None, secondary_dns: None }))), Stopped)
I (8501) rust_esp32_std_demo: Wifi connected, about to do some pings
I (8511) esp_idf_svc::ping: About to run a summary ping 192.168.10.1 with configuration Configuration { count: 5, interval: 1s, timeout: 1s, data_size: 56, tos: 0 }
I (8521) esp_idf_svc::ping: Ping session established, got handle 0x3ffc767c
I (8531) esp_idf_svc::ping: Ping session started
I (8531) esp_idf_svc::ping: Waiting for the ping session to complete
I (8541) esp_idf_svc::ping: Ping success callback invoked
I (8551) esp_idf_svc::ping: From 192.168.10.1 icmp_seq=1 ttl=64 time=14ms bytes=64
I (9531) esp_idf_svc::ping: Ping success callback invoked
I (9531) esp_idf_svc::ping: From 192.168.10.1 icmp_seq=2 ttl=64 time=1ms bytes=64
I (10531) esp_idf_svc::ping: Ping success callback invoked
I (10531) esp_idf_svc::ping: From 192.168.10.1 icmp_seq=3 ttl=64 time=2ms bytes=64
I (11531) esp_idf_svc::ping: Ping success callback invoked
I (11531) esp_idf_svc::ping: From 192.168.10.1 icmp_seq=4 ttl=64 time=0ms bytes=64
I (12531) esp_idf_svc::ping: Ping success callback invoked
I (12531) esp_idf_svc::ping: From 192.168.10.1 icmp_seq=5 ttl=64 time=1ms bytes=64
I (13531) esp_idf_svc::ping: Ping end callback invoked
I (13531) esp_idf_svc::ping: 5 packets transmitted, 5 received, time 18ms
I (13531) esp_idf_svc::ping: Ping session stopped
I (13531) esp_idf_svc::ping: Ping session 0x3ffc767c removed
I (13541) rust_esp32_std_demo: Pinging done
I (13551) esp_idf_svc::httpd: Started Httpd IDF server with config Configuration { http_port: 80, https_port: 443 }
I (13561) esp_idf_svc::httpd: Registered Httpd IDF server handler Get for URI "/"
I (13561) esp_idf_svc::httpd: Registered Httpd IDF server handler Get for URI "/foo"
I (13571) esp_idf_svc::httpd: Registered Httpd IDF server handler Get for URI "/bar"
```
