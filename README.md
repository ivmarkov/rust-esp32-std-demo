# Rust on ESP with STD "Hello, World" app

A small demo app demonstrating calling into STD code (threads, console and TCP/IP).

For more information, check the [STD-enabled Rust compiler port for ESP](https://github.com/ivmarkov/rust).

## Background

The app is a simple "Hello, World" web server implememented with Rust's [Rocket web framework](https://rocket.rs/).

## Building

* The app currently uses PlatformIO as a driver for building the ESP-IDF framework, for linking with the Rust code and for flashing etc.
* The Rust code is compiled to a static C library, which is then linked against the PlatformIO code to get the final elf & bin executables
* Since as per above PlatformIO is only used to build the ESP-IDF framework itself, it might be removed as a requirement in future (even though integration with PlatformIO brings other benefits, like eaiser support for multi-language (C & Rust) projects)
* Since the work to integrate the Rust build system with PlatformIO's own build is unfinished, you'll have to first build the Rust side (using the regular `cargo build --release`) and then trigger the PlatformIO build separately.

### Rough steps

* Clone and build the [Xtensa Rust compiler](https://github.com/ivmarkov/rust) first and make sure it is linked as a custom toolchain in Rustup (TBD: details)
* Clone this repo `git clone https://github.com/ivmarkov/rust-esp32-std-hello`
* `cd rust-esp32-std-hello/rust`
* `cargo build --release`
* `cd ..`
* Apply a small fix to the ESP-IDF TLS pthread support (to be submitted as an issue against the ESP-IDF repo):
```
cd ~/.platformio/packages/framework-espidf
git apply ~/...(this path is specific to your env).../rust-esp32-std-hello/pthread_destructor_fix.diff
cd ~/...(this path is specific to your env).../rust-esp32-std-hello
```
* Change lines 137 and 138 in `rust-esp32-std-hello/rust/src/lib.rs` to contain the SSID & password of your wireless network
* Invoke the PlatformIO build in the app home directory
* Flash

NOTE 1: The debug build is currently VERY large (to be investigated), hence why the steps above produce a release build above

NOTE 2: Even with release build, the final executable will be ~ 1.5MB (where 600-700K are because Rocket is relatively heavy; the other fatness comes from the ESP-IDF WiFi driver itself) which is above the standard app partition size of most ESP boards. Hence **this project has a custom partition**

## Running

* Once you flash and run the app, connect to the board UART0 port, e.g. `miniterm /dev/ttyUSB0 115200` or similar
* You should see more or less the following:

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
I (9019) wifi:wifi driver task: 3ffc1a84, prio:23, stack:6656, core=0
␛[0;32mI (9019) system_api: Base MAC address is not set, read default base MAC address from BLK0 of EFUSE␛[0m
␛[0;32mI (9019) system_api: Base MAC address is not set, read default base MAC address from BLK0 of EFUSE␛[0m
I (9029) wifi:wifi firmware version: 3ea4c76
I (9029) wifi:config NVS flash: disabled
I (9039) wifi:config nano formating: disabled
I (9039) wifi:Init dynamic tx buffer num: 32
I (9049) wifi:Init data frame dynamic rx buffer num: 32
I (9049) wifi:Init management frame dynamic rx buffer num: 32
I (9059) wifi:Init management short buffer num: 32
I (9059) wifi:Init static rx buffer size: 1600
I (9069) wifi:Init static rx buffer num: 10
I (9069) wifi:Init dynamic rx buffer num: 32
␛[0;32mI (9169) phy: phy_version: 4180, cb3948e, Sep 12 2019, 16:39:13, 0, 0␛[0m
I (9169) wifi:mode : sta (f0:08:d1:77:68:f0)
Igniting Rocket...
␛[34m🔧 Configured for production.␛[0m
    ␛[1;49;39m=>␛[0m ␛[34maddress: ␛[1;49;39m0.0.0.0␛[0m␛[34m␛[0m
    ␛[1;49;39m=>␛[0m ␛[34mport: ␛[1;49;39m8000␛[0m␛[34m␛[0m
    ␛[1;49;39m=>␛[0m ␛[34mlog: ␛[1;49;39mcritical␛[0m␛[34m␛[0m
    ␛[1;49;39m=>␛[0m ␛[34mworkers: ␛[1;49;39m2␛[0m␛[34m␛[0m
    ␛[1;49;39m=>␛[0m ␛[34msecret key: ␛[1;49;39mgenerated␛[0m␛[34m␛[0m
    ␛[1;49;39m=>␛[0m ␛[34mlimits: ␛[1;49;39mforms = 32KiB␛[0m␛[34m␛[0m
    ␛[1;49;39m=>␛[0m ␛[34mkeep-alive: ␛[1;49;39m5s␛[0m␛[34m␛[0m
    ␛[1;49;39m=>␛[0m ␛[34mread timeout: ␛[1;49;39m5s␛[0m␛[34m␛[0m
    ␛[1;49;39m=>␛[0m ␛[34mwrite timeout: ␛[1;49;39m5s␛[0m␛[34m␛[0m
    ␛[1;49;39m=>␛[0m ␛[34mtls: ␛[1;49;39mdisabled␛[0m␛[34m␛[0m
␛[1;33mWarning:␛[0m ␛[33menvironment is 'production', but no `secret_key` is configured␛[0m
␛[34m🚀 ␛[1;49;39mRocket has launched from␛[0m␛[34m ␛[1;4;49;39mhttp://␛[0m␛[34m␛[1;4;49;39m0.0.0.0:8000␛[0m␛[34m␛[0m
I (9299) wifi:new:<1,0>, old:<1,0>, ap:<255,255>, sta:<1,0>, prof:1
I (10069) wifi:state: init -> auth (b0)
I (10079) wifi:state: auth -> assoc (0)
I (10079) wifi:state: assoc -> run (10)
I (10099) wifi:connected with ***your-ssid-here***, aid = 1, channel 1, BW20, bssid = ...
I (10099) wifi:security: WPA2-PSK, phy: bgn, rssi: -31
I (10099) wifi:pm start, type: 1

I (10179) wifi:AP's beacon interval = 102400 us, DTIM period = 3
␛[0;32mI (12479) esp_netif_handlers: sta ip: ***the ESP board IP is here***, mask: 255..., gw: ***your-gateway***␛[0m
```

* NOTE: If you have not applied the pthread patch correctly, the app will CRASH just after the line which says "About to join the threads. If ESP-IDF was patched successfully, joining will NOT crash".
* If the app starts successfully, it should be listening on the printed IP address.
* Open a browser, and navigate to `http://<printed-ip-address>:8000/`

