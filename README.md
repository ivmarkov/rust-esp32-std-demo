# Rust on ESP32 "Hello, World" app

A demo binary crate for the ESP32 and ESP-IDF, which connects to WiFi, drives a small HTTP server and draws on a LED screen.

Highlights:
* **Pure Rust and pure Cargo build!** No CMake, no PlatformIO, no C helpers
  * ... via [esp-idf-sys](https://github.com/ivmarkov/esp-idf-sys) and [cargo-pio](https://github.com/ivmarkov/cargo-pio)
* **Support for Rust STD** (threads, console, TCP/IP) safe APIs
  * ... implemented directly in the [Rust ESP32 STD compiler fork](https://github.com/ivmarkov/rus)
* Rust Safe APIs for various ESP-IDF services like WiFi, Ping, Httpd and logging
  * ... via [esp-idf-svc](https://github.com/ivmarkov/esp-idf-svc) ([embedded-svc](https://github.com/ivmarkov/embedded-svc) abstractions implemented on top of ESP-IDF)
* Driving a LED screen with the [embedded-graphics](https://github.com/embedded-graphics/embedded-graphics) Rust crate
  * via [esp-idf-hal](https://github.com/ivmarkov/esp-idf-hal) ([embedded-hal](https://github.com/rust-embedded/embedded-hal) drivers implemented on top of ESP-IDF)

## Build


* Install the nightly toolchain of Rust (necessary, because we utilize a few unstable Cargo features): ```rustup toolchain install nightly```
* Make sure the toolchains are up to date, as one of the utilized unstable Cargo features landed just a few months ago: ```rustup update```
* Switch to nightly (as per above, necessary for Cargo): ```rustup default nightly```
* Download and install the [prebuilt binaries of the Rust ESP32 STD compiler fork](https://github.com/espressif/rust-esp32-example/blob/main/docs/rust-on-xtensa.md) or follow the [Rust ESP32 STD compiler fork build instructions](https://github.com/ivmarkov/rust);
* Download and install the ESP32 GCC toolchain for your chip (ESP32, ESP32S2, ESP32C3). **You can utilize [cargo-pio](https://github.com/ivmarkov/cargo-pio) for that**:
  * ```cargo install cargo-pio --git https://github.com/ivmarkov/cargo-pio```
  * ```cargo pio installpio```
  * ```cargo pio printscons --var path --frameworks espidf --platform espressif32 --mcu [ESP32|ESP32S2|ESP32C3]```
    * Please specify *ONE* of ESP32, ESP32S2 or ESP32C3 in the command line above and below
  * ```export PATH=`cargo pio -q printscons --var path --frameworks espidf --platform espressif32 --mcu [ESP32|ESP32S2|ESP32C3]`:$PATH```
* Clone this repo: ```git clone https://github.com/ivmarkov/rust-esp32-std-hello```
* Enter it: ```cd rust-esp32-std-hello/rust```
* Change lines 295 and 296 in `rust-esp32-std-hello/rust/src/main.rs` to contain the SSID & password of your wireless network
* Build: ```cargo build``` or ```cargo build --release```

## Flash

* ```cargo install espflash```
* ```espflash /dev/ttyUSB0 target/xtensa-esp32-none/debug/rust-esp32-std-hello```

## Monitor

* Once you flash and run the app, connect to the board UART0 port, e.g. ```miniterm --raw /dev/ttyUSB0 115200``` or similar
* You should see more or less the following:

```
Hello, world from Rust!
More complex print [foo, bar]
Calling into C to get The Answer: 42. Big surprise.
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
I (4831) rust_esp32_std_hello: Wifi created
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
I (8471) rust_esp32_std_hello: Wifi configuration set, about to get status
I (8481) esp_idf_svc::wifi: Providing status: Status(Started(Connected(Done(ClientSettings { ip: 192.168.10.155, subnet: Subnet { gateway: 192.168.10.1, mask: Mask(24) }, dns: None, secondary_dns: None }))), Stopped)
I (8501) rust_esp32_std_hello: Wifi connected, about to do some pings
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
I (13541) rust_esp32_std_hello: Pinging done
I (13551) esp_idf_svc::httpd: Started Httpd IDF server with config Configuration { http_port: 80, https_port: 443 }
I (13561) esp_idf_svc::httpd: Registered Httpd IDF server handler Get for URI "/"
I (13561) esp_idf_svc::httpd: Registered Httpd IDF server handler Get for URI "/foo"
I (13571) esp_idf_svc::httpd: Registered Httpd IDF server handler Get for URI "/bar"
```

* NOTE: If you have not applied the pthread patch correctly, the app will CRASH just after the line which says "About to join the threads. If ESP-IDF was patched successfully, joining will NOT crash".
* If the app starts successfully, it should be listening on the printed IP address from the WiFi connection logs, port 80.
* Open a browser, and navigate to one of these:
* `http://<printed-ip-address>`
* `http://<printed-ip-address>/foo?key=value`
* `http://<printed-ip-address>/bar`
