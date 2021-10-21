#![allow(unused_imports)]
#![allow(clippy::single_component_path_imports)]
//#![feature(backtrace)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Condvar, Mutex};
use std::{cell::RefCell, env, sync::atomic::*, sync::Arc, thread, time::*};

use anyhow::*;
use log::*;

use url;

use smol;

use embedded_svc::anyerror::*;
use embedded_svc::eth;
use embedded_svc::eth::Eth;
use embedded_svc::httpd::registry::*;
use embedded_svc::httpd::*;
use embedded_svc::io;
use embedded_svc::ipv4;
use embedded_svc::ping::Ping;
use embedded_svc::wifi::*;

use esp_idf_svc::eth::*;
use esp_idf_svc::httpd as idf;
use esp_idf_svc::httpd::ServerRegistry;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;

use esp_idf_hal::delay;
use esp_idf_hal::gpio;
use esp_idf_hal::i2c;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;
use esp_idf_hal::ulp;

use esp_idf_sys;
use esp_idf_sys::esp;

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;

use ili9341;
use ssd1306;
use ssd1306::mode::DisplayConfig;
use st7789;

#[allow(dead_code)]
const SSID: &str = "ssid";
#[allow(dead_code)]
const PASS: &str = "pass";

#[cfg(esp32s2)]
include!(env!("EMBUILD_GENERATED_SYMBOLS_FILE"));

#[cfg(esp32s2)]
const ULP: &[u8] = include_bytes!(env!("EMBUILD_GENERATED_BIN_FILE"));

thread_local! {
    static TLS: RefCell<u32> = RefCell::new(13);
}

fn main() -> Result<()> {
    test_print();

    test_atomics();

    test_threads();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Get backtraces from anyhow; only works for Xtensa arch currently
    #[cfg(arch = "xtensa")]
    env::set_var("RUST_BACKTRACE", "1");

    // Uncomment this if you have a TTGO ESP32 board
    // For other boards, you might have to use a different embedded-graphics driver and pin configuration
    // ttgo_hello_world()?;

    // ... or uncomment this if you have a Kaluga-1 ESP32-S2 board
    // For other boards, you might have to use a different embedded-graphics driver and pin configuration
    // #[cfg(esp32s2)]
    // kaluga_hello_world(true)?;

    // ... or uncomment this if you have a ESP32-S3-OTG-USB (including S2) board
    // #[cfg(esp32s2)]
    // esp32s3_usb_otg_hello_world()?;

    // ... or uncomment this if you have a Heltec LoRa 32 board
    // For other boards, you might have to use a different embedded-graphics driver and pin configuration
    // heltec_hello_world()?;

    #[cfg(not(feature = "qemu"))]
    #[allow(unused_mut)]
    let mut wifi = wifi()?;

    #[cfg(feature = "qemu")]
    let eth = eth_qemu()?;

    test_tcp()?;

    #[cfg(feature = "bind")]
    test_tcp_bind()?;

    #[cfg(all(feature = "bind", esp_idf_version = "4.4"))]
    test_tcp_bind_async()?;

    #[cfg(feature = "experimental")]
    test_https_client()?;

    #[cfg(not(feature = "qemu"))]
    #[cfg(esp_idf_config_lwip_ipv4_napt)]
    enable_napt(&mut wifi)?;

    let mutex = Arc::new((Mutex::new(None), Condvar::new()));

    let httpd = httpd(mutex.clone())?;

    let mut wait = mutex.0.lock().unwrap();

    #[allow(unused)]
    let cycles = loop {
        if let Some(cycles) = *wait {
            break cycles;
        } else {
            wait = mutex.1.wait(wait).unwrap();
        }
    };

    for s in 0..3 {
        info!("Shutting down in {} secs", 3 - s);
        thread::sleep(Duration::from_secs(1));
    }

    drop(httpd);
    info!("Httpd stopped");

    #[cfg(not(feature = "qemu"))]
    {
        drop(wifi);
        info!("Wifi stopped");
    }

    #[cfg(feature = "qemu")]
    {
        let _hw = eth.release()?;
        info!("Eth stopped");
    }

    #[cfg(esp32s2)]
    start_ulp(cycles)?;

    Ok(())
}

#[allow(clippy::vec_init_then_push)]
fn test_print() {
    // Start simple
    println!("Hello, world from Rust!");

    // Check collections
    let mut children = vec![];

    children.push("foo");
    children.push("bar");
    println!("More complex print {:?}", children);
}

#[allow(deprecated)]
fn test_atomics() {
    let a = AtomicUsize::new(0);
    let v1 = a.compare_and_swap(0, 1, Ordering::SeqCst);
    let v2 = a.swap(2, Ordering::SeqCst);

    let (r1, r2) = unsafe {
        // don't optimize our atomics out
        let r1 = core::ptr::read_volatile(&v1);
        let r2 = core::ptr::read_volatile(&v2);

        (r1, r2)
    };

    println!("Result: {}, {}", r1, r2);
}

fn test_threads() {
    let mut children = vec![];

    println!("Rust main thread: {:?}", thread::current());

    TLS.with(|tls| {
        println!("Main TLS before change: {}", *tls.borrow());
    });

    TLS.with(|tls| *tls.borrow_mut() = 42);

    TLS.with(|tls| {
        println!("Main TLS after change: {}", *tls.borrow());
    });

    for i in 0..5 {
        // Spin up another thread
        children.push(thread::spawn(move || {
            println!("This is thread number {}, {:?}", i, thread::current());

            TLS.with(|tls| *tls.borrow_mut() = i);

            TLS.with(|tls| {
                println!("Inner TLS: {}", *tls.borrow());
            });
        }));
    }

    println!(
        "About to join the threads. If ESP-IDF was patched successfully, joining will NOT crash"
    );

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }

    TLS.with(|tls| {
        println!("Main TLS after threads: {}", *tls.borrow());
    });

    thread::sleep(Duration::from_secs(2));

    println!("Joins were successful.");
}

fn test_tcp() -> Result<()> {
    info!("About to open a TCP connection to 1.1.1.1 port 80");

    let mut stream = TcpStream::connect("one.one.one.one:80")?;

    let err = stream.try_clone();
    if let Err(err) = err {
        info!(
            "Duplication of file descriptors does not work (yet) on the ESP-IDF, as expected: {}",
            err
        );
    }

    stream.write_all("GET / HTTP/1.0\n\n".as_bytes())?;

    let mut result = Vec::new();

    stream.read_to_end(&mut result)?;

    info!(
        "1.1.1.1 returned:\n=================\n{}\n=================\nSince it returned something, all is OK",
        std::str::from_utf8(&result)?);

    Ok(())
}

#[cfg(feature = "bind")]
fn test_tcp_bind() -> Result<()> {
    fn test_tcp_bind_accept() -> Result<()> {
        info!("About to bind a simple echo service to port 8080");

        let listener = TcpListener::bind("0.0.0.0:8080")?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    info!("Accepted client");

                    thread::spawn(move || {
                        test_tcp_bind_handle_client(stream);
                    });
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }

        unreachable!()
    }

    fn test_tcp_bind_handle_client(mut stream: TcpStream) {
        // read 20 bytes at a time from stream echoing back to stream
        loop {
            let mut read = [0; 128];

            match stream.read(&mut read) {
                Ok(n) => {
                    if n == 0 {
                        // connection was closed
                        break;
                    }
                    stream.write(&read[0..n]).unwrap();
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    }

    thread::spawn(|| test_tcp_bind_accept().unwrap());

    Ok(())
}

#[cfg(all(feature = "bind", esp_idf_version = "4.4"))]
fn test_tcp_bind_async() -> anyhow::Result<()> {
    async fn test_tcp_bind() -> smol::io::Result<()> {
        /// Echoes messages from the client back to it.
        async fn echo(stream: smol::Async<TcpStream>) -> smol::io::Result<()> {
            smol::io::copy(&stream, &mut &stream).await?;
            Ok(())
        }

        // Create a listener.
        let listener = smol::Async::<TcpListener>::bind(([0, 0, 0, 0], 8081))?;

        // Accept clients in a loop.
        loop {
            let (stream, peer_addr) = listener.accept().await?;
            info!("Accepted client: {}", peer_addr);

            // Spawn a task that echoes messages from the client back to it.
            smol::spawn(echo(stream)).detach();
        }
    }

    info!("About to bind a simple echo service to port 8081 using async (smol-rs)!");

    esp_idf_sys::esp!(unsafe {
        esp_idf_sys::esp_vfs_eventfd_register(&esp_idf_sys::esp_vfs_eventfd_config_t {
            max_fds: 5,
            ..Default::default()
        })
    })?;

    thread::spawn(move || {
        smol::block_on(test_tcp_bind()).unwrap();
    });

    Ok(())
}

#[cfg(feature = "experimental")]
fn test_https_client() -> Result<()> {
    use embedded_svc::http::{self, client::*, status, HttpHeaders, HttpStatus};
    use esp_idf_svc::http::client::*;

    let url = String::from("https://google.com");

    info!("About to fetch content from {}", url);

    let mut client = EspHttpClient::new_default()?;

    let response = client.get(&url)?.submit()?;

    let mut body = Vec::new();
    io::StdIO(response.into_payload())
        .take(3084)
        .read_to_end(&mut body)?;

    info!(
        "Body (truncated to 3K):\n{:?}",
        String::from_utf8_lossy(&body).into_owned()
    );

    Ok(())
}

#[allow(dead_code)]
#[cfg(esp32)]
fn ttgo_hello_world() -> Result<()> {
    info!("About to initialize the TTGO ST7789 LED driver");

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let config = <spi::config::Config as Default>::default()
        .baudrate(26.MHz().into())
        .bit_order(spi::config::BitOrder::MSBFirst);

    let mut backlight = pins.gpio4.into_output()?;
    backlight.set_high()?;

    let di = SPIInterfaceNoCS::new(
        spi::Master::<spi::SPI2, _, _, _, _>::new(
            peripherals.spi2,
            spi::Pins {
                sclk: pins.gpio18,
                sdo: pins.gpio19,
                sdi: Option::<gpio::Gpio21<gpio::Unknown>>::None,
                cs: Some(pins.gpio5),
            },
            config,
        )?,
        pins.gpio16.into_output()?,
    );

    let mut display = st7789::ST7789::new(
        di,
        pins.gpio23.into_output()?,
        // SP7789V is designed to drive 240x320 screens, even though the TTGO physical screen is smaller
        240,
        320,
    );

    AnyError::<st7789::Error<_>>::wrap(|| {
        display.init(&mut delay::Ets)?;
        display.set_orientation(st7789::Orientation::Portrait)?;

        // The TTGO board's screen does not start at offset 0x0, and the physical size is 135x240, instead of 240x320
        let top_left = Point::new(52, 40);
        let size = Size::new(135, 240);

        led_draw(&mut display.cropped(&Rectangle::new(top_left, size)))
    })
}

#[allow(dead_code)]
#[cfg(esp32s2)]
fn kaluga_hello_world(ili9341: bool) -> Result<()> {
    info!(
        "About to initialize the Kaluga {} SPI LED driver",
        if ili9341 { "ILI9341" } else { "ST7789" }
    );

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let config = <spi::config::Config as Default>::default()
        .baudrate((if ili9341 { 40 } else { 80 }).MHz().into())
        .bit_order(spi::config::BitOrder::MSBFirst);

    let mut backlight = pins.gpio6.into_output()?;
    backlight.set_high()?;

    let di = SPIInterfaceNoCS::new(
        spi::Master::<spi::SPI3, _, _, _, _>::new(
            peripherals.spi3,
            spi::Pins {
                sclk: pins.gpio15,
                sdo: pins.gpio9,
                sdi: Option::<gpio::Gpio21<gpio::Unknown>>::None,
                cs: Some(pins.gpio11),
            },
            config,
        )?,
        pins.gpio13.into_output()?,
    );

    let reset = pins.gpio16.into_output()?;

    if ili9341 {
        AnyError::<ili9341::DisplayError>::wrap(|| {
            let mut display = ili9341::Ili9341::new(
                di,
                reset,
                &mut delay::Ets,
                KalugaOrientation::Landscape,
                ili9341::DisplaySize240x320,
            )?;

            led_draw(&mut display)
        })
    } else {
        let mut display = st7789::ST7789::new(di, reset, 320, 240);

        AnyError::<st7789::Error<_>>::wrap(|| {
            display.init(&mut delay::Ets)?;
            display.set_orientation(st7789::Orientation::Landscape)?;

            led_draw(&mut display)
        })
    }
}

#[allow(dead_code)]
#[cfg(esp32s2)]
fn esp32s3_usb_otg_hello_world() -> Result<()> {
    info!(
        "About to initialize the ESP32-S3-USB-OTG SPI LED driver ST7789VW"
    );

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let config = <spi::config::Config as Default>::default()
        .baudrate(80.MHz().into())
        .bit_order(spi::config::BitOrder::MSBFirst);

    let mut backlight = pins.gpio9.into_output()?;
    backlight.set_high()?;

    let di = SPIInterfaceNoCS::new(
        spi::Master::<spi::SPI3, _, _, _, _>::new(
            peripherals.spi3,
            spi::Pins {
                sclk: pins.gpio6,
                sdo: pins.gpio7,
                sdi: Option::<gpio::Gpio21<gpio::Unknown>>::None,
                cs: Some(pins.gpio5),
            },
            config,
        )?,
        pins.gpio4.into_output()?,
    );

    let reset = pins.gpio8.into_output()?;

    let mut display = st7789::ST7789::new(di, reset, 240, 240);

    AnyError::<st7789::Error<_>>::wrap(|| {
        display.init(&mut delay::Ets)?;
        display.set_orientation(st7789::Orientation::Landscape)?;

        led_draw(&mut display)
    })
}

#[allow(dead_code)]
#[cfg(esp32)]
fn heltec_hello_world() -> Result<()> {
    info!("About to initialize the Heltec SSD1306 I2C LED driver");

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let config = <i2c::config::MasterConfig as Default>::default().baudrate(400.kHz().into());

    let di = ssd1306::I2CDisplayInterface::new(i2c::Master::<i2c::I2C0, _, _>::new(
        peripherals.i2c0,
        i2c::Pins {
            sda: pins.gpio4,
            scl: pins.gpio15,
        },
        config,
    )?);

    let mut delay = delay::Ets;
    let mut reset = pins.gpio16.into_output()?;

    reset.set_high()?;
    delay.delay_ms(1 as u32);

    reset.set_low()?;
    delay.delay_ms(10 as u32);

    reset.set_high()?;

    let mut display = Box::new(
        ssd1306::Ssd1306::new(
            di,
            ssd1306::size::DisplaySize128x64,
            ssd1306::rotation::DisplayRotation::Rotate0,
        )
        .into_buffered_graphics_mode(),
    );

    AnyError::<display_interface::DisplayError>::wrap(|| {
        display.init()?;

        led_draw(&mut *display)?;

        display.flush()
    })
}

#[allow(dead_code)]
fn led_draw<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget + Dimensions,
    D::Color: From<Rgb565>,
{
    display.clear(Rgb565::BLACK.into())?;

    Rectangle::new(display.bounding_box().top_left, display.bounding_box().size)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLUE.into())
                .stroke_color(Rgb565::YELLOW.into())
                .stroke_width(1)
                .build(),
        )
        .draw(display)?;

    Text::new(
        "Hello Rust!",
        Point::new(10, (display.bounding_box().size.height - 10) as i32 / 2),
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE.into()),
    )
    .draw(display)?;

    info!("LED rendering done");

    Ok(())
}

#[allow(unused_variables)]
fn httpd(mutex: Arc<(Mutex<Option<u32>>, Condvar)>) -> Result<idf::Server> {
    let server = idf::ServerRegistry::new()
        .at("/")
        .get(|_| Ok("Hello, world!".into()))?
        .at("/foo")
        .get(|_| bail!("Boo, something happened!"))?
        .at("/bar")
        .get(|_| {
            Response::new(403)
                .status_message("No permissions")
                .body("You have no permissions to access this page".into())
                .into()
        })?;

    #[cfg(esp32s2)]
    let server = httpd_ulp_endpoints(server, mutex)?;

    server.start(&Default::default())
}

#[cfg(esp32s2)]
fn httpd_ulp_endpoints(
    server: ServerRegistry,
    mutex: Arc<(Mutex<Option<u32>>, Condvar)>,
) -> Result<ServerRegistry> {
    server
        .at("/ulp")
        .get(|_| {
            Ok(r#"
            <doctype html5>
            <html>
                <body>
                    <form method = "post" action = "/ulp_start" enctype="application/x-www-form-urlencoded">
                        Connect a LED to ESP32-S2 GPIO <b>Pin 04</b> and GND.<br>
                        Blink it with ULP <input name = "cycles" type = "text" value = "10"> times
                        <input type = "submit" value = "Go!">
                    </form>
                </body>
            </html>
            "#.into())
        })?
        .at("/ulp_start")
        .post(move |mut request| {
            let body = request.as_bytes()?;

            let cycles = url::form_urlencoded::parse(&body)
                .filter(|p| p.0 == "cycles")
                .map(|p| str::parse::<u32>(&p.1).map_err(Error::msg))
                .next()
                .ok_or(anyhow!("No parameter cycles"))??;

            let mut wait = mutex.0.lock().unwrap();

            *wait = Some(cycles);
            mutex.1.notify_one();

            Ok(format!(
                r#"
                <doctype html5>
                <html>
                    <body>
                        About to sleep now. The ULP chip should blink the LED {} times and then wake me up. Bye!
                    </body>
                </html>
                "#,
                cycles).to_owned().into())
        })
}

#[cfg(esp32s2)]
fn start_ulp(cycles: u32) -> Result<()> {
    use esp_idf_hal::ulp;

    unsafe {
        esp!(esp_idf_sys::ulp_riscv_load_binary(
            ULP.as_ptr(),
            ULP.len() as _
        ))?;
        info!("RiscV ULP binary loaded successfully");

        // Once started, the ULP will wakeup every 5 minutes
        // TODO: Figure out how to disable ULP timer-based wakeup completely, with an ESP-IDF call
        ulp::enable_timer(false);

        info!("RiscV ULP Timer configured");

        info!(
            "Default ULP LED blink cycles: {}",
            core::ptr::read_volatile(CYCLES as *mut u32)
        );

        core::ptr::write_volatile(CYCLES as *mut u32, cycles);
        info!(
            "Sent {} LED blink cycles to the ULP",
            core::ptr::read_volatile(CYCLES as *mut u32)
        );

        esp!(esp_idf_sys::ulp_riscv_run())?;
        info!("RiscV ULP started");

        esp!(esp_idf_sys::esp_sleep_enable_ulp_wakeup())?;
        info!("Wakeup from ULP enabled");

        // Wake up by a timer in 60 seconds
        info!("About to get to sleep now. Will wake up automatically either in 1 minute, or once the ULP has done blinking the LED");
        esp_idf_sys::esp_deep_sleep(Duration::from_secs(60).as_micros() as u64);
    }

    Ok(())
}

#[cfg(not(feature = "qemu"))]
fn wifi() -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(
        Arc::new(EspNetifStack::new()?),
        Arc::new(EspSysLoopStack::new()?),
        Arc::new(EspDefaultNvs::new()?),
    )?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    info!("Wifi configuration set, about to get status");

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected");

        ping(&ip_settings)?;
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

#[cfg(feature = "qemu")]
fn eth_qemu() -> Result<Box<EspEth<()>>> {
    let mut eth = Box::new(EspEth::new_openeth(
        Arc::new(EspNetifStack::new()?),
        Arc::new(EspSysLoopStack::new()?),
    )?);

    info!("Eth created");

    eth.set_configuration(&eth::Configuration::Client(Default::default()))?;

    info!("Eth configuration set, about to get status");

    let status = eth.get_status();

    if let eth::Status::Started(eth::ConnectionStatus::Connected(eth::IpStatus::Done(Some(
        ip_settings,
    )))) = status
    {
        info!("Eth connected");

        ping(&ip_settings)?;
    } else {
        bail!("Unexpected Eth status: {:?}", status);
    }

    Ok(eth)
}

fn ping(ip_settings: &ipv4::ClientSettings) -> Result<()> {
    info!("About to do some pings for {:?}", ip_settings);

    let ping_summary =
        ping::EspPing::default().ping(ip_settings.subnet.gateway, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!(
            "Pinging gateway {} resulted in timeouts",
            ip_settings.subnet.gateway
        );
    }

    info!("Pinging done");

    Ok(())
}

#[cfg(not(feature = "qemu"))]
#[cfg(esp_idf_config_lwip_ipv4_napt)]
fn enable_napt(wifi: &mut EspWifi) -> Result<()> {
    wifi.with_router_netif_mut(|netif| netif.unwrap().enable_napt(true));

    info!("NAPT enabled on the WiFi SoftAP!");

    Ok(())
}

// Kaluga needs customized screen orientation commands
// (not a surprise; quite a few ILI9341 boards need these as evidences in the TFT_eSPI & lvgl ESP32 C drivers)
pub enum KalugaOrientation {
    Portrait,
    PortraitFlipped,
    Landscape,
    LandscapeFlipped,
}

impl ili9341::Mode for KalugaOrientation {
    fn mode(&self) -> u8 {
        match self {
            Self::Portrait => 0,
            Self::Landscape => 0x20 | 0x40,
            Self::PortraitFlipped => 0x80 | 0x40,
            Self::LandscapeFlipped => 0x80 | 0x20,
        }
    }

    fn is_landscape(&self) -> bool {
        matches!(self, Self::Landscape | Self::LandscapeFlipped)
    }
}
