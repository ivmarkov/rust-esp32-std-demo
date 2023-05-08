#![allow(unused_imports)]
#![allow(clippy::single_component_path_imports)]
//#![feature(backtrace)]

#[cfg(all(feature = "qemu", not(esp32)))]
compile_error!("The `qemu` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "ip101", not(esp32)))]
compile_error!("The `ip101` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "kaluga", not(esp32s2)))]
compile_error!("The `kaluga` feature can only be built for the `xtensa-esp32s2-espidf` target.");

#[cfg(all(feature = "ttgo", not(esp32)))]
compile_error!("The `ttgo` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "heltec", not(esp32)))]
compile_error!("The `heltec` feature can only be built for the `xtensa-esp32-espidf` target.");

#[cfg(all(feature = "esp32s3_usb_otg", not(esp32s3)))]
compile_error!(
    "The `esp32s3_usb_otg` feature can only be built for the `xtensa-esp32s3-espidf` target."
);

use core::ffi;

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Condvar, Mutex};
use std::{cell::RefCell, env, sync::atomic::*, sync::Arc, thread, time::*};

use anyhow::bail;

use log::*;

use url;

use smol;

use embedded_hal::adc::OneShot;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;

use embedded_svc::eth;
#[allow(deprecated)]
use embedded_svc::httpd::{registry::*, *};
use embedded_svc::io;
use embedded_svc::ipv4;
use embedded_svc::mqtt::client::{Client, Connection, MessageImpl, Publish, QoS};
use embedded_svc::ping::Ping;
use embedded_svc::sys_time::SystemTime;
use embedded_svc::timer::TimerService;
use embedded_svc::timer::*;
use embedded_svc::utils::mqtt::client::ConnState;
use embedded_svc::wifi::*;

use esp_idf_svc::eventloop::*;
use esp_idf_svc::httpd as idf;
use esp_idf_svc::httpd::ServerRegistry;
use esp_idf_svc::mqtt::client::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sntp;
use esp_idf_svc::systime::EspSystemTime;
use esp_idf_svc::timer::*;
use esp_idf_svc::wifi::*;

use esp_idf_hal::adc;
use esp_idf_hal::delay;
use esp_idf_hal::gpio;
use esp_idf_hal::i2c;
use esp_idf_hal::peripheral;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;

use esp_idf_sys;
use esp_idf_sys::{esp, EspError};

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;

use mipidsi;
use ssd1306;
use ssd1306::mode::DisplayConfig;

use epd_waveshare::{epd4in2::*, graphics::VarDisplay, prelude::*};

#[allow(dead_code)]
#[cfg(not(feature = "qemu"))]
const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
#[allow(dead_code)]
#[cfg(not(feature = "qemu"))]
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

#[cfg(esp32s2)]
include!(env!("EMBUILD_GENERATED_SYMBOLS_FILE"));

#[cfg(esp32s2)]
const ULP: &[u8] = include_bytes!(env!("EMBUILD_GENERATED_BIN_FILE"));

thread_local! {
    static TLS: RefCell<u32> = RefCell::new(13);
}

static CS: esp_idf_hal::task::CriticalSection = esp_idf_hal::task::CriticalSection::new();

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    test_print();

    test_atomics();

    test_threads();

    #[cfg(not(esp_idf_version = "4.3"))]
    test_fs()?;

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Get backtraces from anyhow; only works for Xtensa arch currently
    // TODO: No longer working with ESP-IDF 4.3.1+
    //#[cfg(target_arch = "xtensa")]
    //env::set_var("RUST_BACKTRACE", "1");

    #[allow(unused)]
    let peripherals = Peripherals::take().unwrap();
    #[allow(unused)]
    let pins = peripherals.pins;

    // If interrupt critical sections work fine, the code below should panic with the IWDT triggering
    // {
    //     info!("Testing interrupt critical sections");

    //     let mut x = 0;

    //     esp_idf_hal::interrupt::free(move || {
    //         for _ in 0..2000000 {
    //             for _ in 0..2000000 {
    //                 x += 1;

    //                 if x == 1000000 {
    //                     break;
    //                 }
    //             }
    //         }
    //     });
    // }

    {
        info!("Testing critical sections");

        {
            let th = {
                let _guard = CS.enter();

                let th = std::thread::spawn(move || {
                    info!("Waiting for critical section");
                    let _guard = CS.enter();

                    info!("Critical section acquired");
                });

                std::thread::sleep(Duration::from_secs(5));

                th
            };

            th.join().unwrap();
        }
    }

    #[allow(unused)]
    let sysloop = EspSystemEventLoop::take()?;

    #[cfg(feature = "ttgo")]
    ttgo_hello_world(
        pins.gpio4,
        pins.gpio16,
        pins.gpio23,
        peripherals.spi2,
        pins.gpio18,
        pins.gpio19,
        pins.gpio5,
    )?;

    #[cfg(feature = "waveshare_epd")]
    waveshare_epd_hello_world(
        peripherals.spi2,
        pins.gpio13.into(),
        pins.gpio14.into(),
        pins.gpio15.into(),
        pins.gpio25.into(),
        pins.gpio27.into(),
        pins.gpio26.into(),
    )?;

    #[cfg(feature = "kaluga")]
    kaluga_hello_world(
        pins.gpio6,
        pins.gpio13,
        pins.gpio16,
        peripherals.spi3,
        pins.gpio15,
        pins.gpio9,
        pins.gpio11,
    )?;

    #[cfg(feature = "heltec")]
    heltec_hello_world(pins.gpio16, peripherals.i2c0, pins.gpio4, pins.gpio15)?;

    #[cfg(feature = "ssd1306g_spi")]
    ssd1306g_hello_world_spi(
        pins.gpio4.into(),
        pins.gpio16.into(),
        peripherals.spi3,
        pins.gpio18.into(),
        pins.gpio23.into(),
        pins.gpio5.into(),
    )?;

    #[cfg(feature = "ssd1306g")]
    let mut led_power = ssd1306g_hello_world(
        peripherals.i2c0,
        pins.gpio14.into(),
        pins.gpio22.into(),
        pins.gpio21.into(),
    )?;

    #[cfg(feature = "esp32s3_usb_otg")]
    esp32s3_usb_otg_hello_world(
        pins.gpio9,
        pins.gpio4,
        pins.gpio8,
        peripherals.spi3,
        pins.gpio6,
        pins.gpio7,
        pins.gpio5,
    )?;

    #[allow(clippy::redundant_clone)]
    #[cfg(not(feature = "qemu"))]
    #[allow(unused_mut)]
    let mut wifi = wifi(peripherals.modem, sysloop.clone())?;

    #[allow(clippy::redundant_clone)]
    #[cfg(feature = "qemu")]
    let eth = eth_configure(
        &sysloop,
        Box::new(esp_idf_svc::eth::EspEth::wrap(
            esp_idf_svc::eth::EthDriver::new_openeth(peripherals.mac, sysloop.clone())?,
        )?),
    )?;

    #[allow(clippy::redundant_clone)]
    #[cfg(feature = "ip101")]
    let eth = eth_configure(
        &sysloop,
        Box::new(esp_idf_svc::eth::EspEth::wrap(
            esp_idf_svc::eth::EthDriver::new_rmii(
                peripherals.mac,
                pins.gpio25,
                pins.gpio26,
                pins.gpio27,
                pins.gpio23,
                pins.gpio22,
                pins.gpio21,
                pins.gpio19,
                pins.gpio18,
                esp_idf_svc::eth::RmiiClockConfig::<gpio::Gpio0, gpio::Gpio16, gpio::Gpio17>::Input(
                    pins.gpio0,
                ),
                Some(pins.gpio5),
                esp_idf_svc::eth::RmiiEthChipset::IP101,
                None,
                sysloop.clone(),
            )?,
        )?),
    )?;

    #[cfg(feature = "w5500")]
    let eth = eth_configure(
        &sysloop,
        Box::new(esp_idf_svc::eth::EspEth::wrap(
            esp_idf_svc::eth::EthDriver::new_spi(
                peripherals.spi2,
                pins.gpio13,
                pins.gpio12,
                pins.gpio26,
                pins.gpio27,
                esp_idf_hal::spi::Dma::Auto(4096),
                Some(pins.gpio14),
                Some(pins.gpio25),
                esp_idf_svc::eth::SpiEthChipset::W5500,
                20.MHz().into(),
                Some(&[0x02, 0x00, 0x00, 0x12, 0x34, 0x56]),
                None,
                sysloop.clone(),
            )?,
        )?),
    )?;

    test_tcp()?;

    test_tcp_bind()?;

    let _sntp = sntp::EspSntp::new_default()?;
    info!("SNTP initialized");

    let (eventloop, _subscription) = test_eventloop()?;

    let mqtt_client = test_mqtt_client()?;

    let _timer = test_timer(eventloop, mqtt_client)?;

    #[cfg(feature = "experimental")]
    experimental::test()?;

    #[cfg(not(feature = "qemu"))]
    #[cfg(esp_idf_lwip_ipv4_napt)]
    enable_napt(&mut wifi)?;

    let mutex = Arc::new((Mutex::new(None), Condvar::new()));

    let httpd = httpd(mutex.clone())?;

    #[cfg(feature = "ssd1306g")]
    {
        for s in 0..3 {
            info!("Powering off the display in {} secs", 3 - s);
            thread::sleep(Duration::from_secs(1));
        }

        led_power.set_low()?;
    }

    let mut wait = mutex.0.lock().unwrap();

    #[cfg(all(esp32, esp_idf_version_major = "4"))]
    let mut hall_sensor = peripherals.hall_sensor;

    #[cfg(esp32)]
    let adc_pin = pins.gpio34;
    #[cfg(not(esp32))]
    let adc_pin = pins.gpio2;

    let mut a2 = adc::AdcChannelDriver::<_, adc::Atten11dB<adc::ADC1>>::new(adc_pin)?;

    let mut powered_adc1 = adc::AdcDriver::new(
        peripherals.adc1,
        &adc::config::Config::new().calibration(true),
    )?;

    #[allow(unused)]
    let cycles = loop {
        if let Some(cycles) = *wait {
            break cycles;
        } else {
            wait = mutex
                .1
                .wait_timeout(wait, Duration::from_secs(1))
                .unwrap()
                .0;

            #[cfg(all(esp32, esp_idf_version_major = "4"))]
            log::info!(
                "Hall sensor reading: {}mV",
                powered_adc1.read_hall(&mut hall_sensor).unwrap()
            );
            log::info!(
                "A2 sensor reading: {}mV",
                powered_adc1.read(&mut a2).unwrap()
            );
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

    #[cfg(any(feature = "qemu", feature = "w5500", feature = "ip101"))]
    {
        drop(eth);
        info!("Eth stopped");
    }

    #[cfg(esp32s2)]
    start_ulp(peripherals.ulp, cycles)?;

    Ok(())
}

#[allow(clippy::vec_init_then_push)]
fn test_print() {
    // Start simple
    println!("Hello from Rust!");

    // Check collections
    let mut children = vec![];

    children.push("foo");
    children.push("bar");
    println!("More complex print {children:?}");
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

    println!("Result: {r1}, {r2}");
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

#[cfg(not(esp_idf_version = "4.3"))]
fn test_fs() -> Result<()> {
    assert_eq!(fs::canonicalize(PathBuf::from("."))?, PathBuf::from("/"));
    assert_eq!(
        fs::canonicalize(
            PathBuf::from("/")
                .join("foo")
                .join("bar")
                .join(".")
                .join("..")
                .join("baz")
        )?,
        PathBuf::from("/foo/baz")
    );

    Ok(())
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
                    stream.write_all(&read[0..n]).unwrap();
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

fn test_timer(
    eventloop: EspBackgroundEventLoop,
    mut client: EspMqttClient<ConnState<MessageImpl, EspError>>,
) -> Result<EspTimer> {
    use embedded_svc::event_bus::Postbox;

    info!("About to schedule a one-shot timer for after 2 seconds");
    let once_timer = EspTimerService::new()?.timer(|| {
        info!("One-shot timer triggered");
    })?;

    once_timer.after(Duration::from_secs(2))?;

    thread::sleep(Duration::from_secs(3));

    info!("About to schedule a periodic timer every five seconds");
    let periodic_timer = EspTimerService::new()?.timer(move || {
        info!("Tick from periodic timer");

        let now = EspSystemTime {}.now();

        eventloop.post(&EventLoopMessage::new(now), None).unwrap();

        client
            .publish(
                "rust-esp32-std-demo",
                QoS::AtMostOnce,
                false,
                format!("Now is {now:?}").as_bytes(),
            )
            .unwrap();
    })?;

    periodic_timer.every(Duration::from_secs(5))?;

    Ok(periodic_timer)
}

#[derive(Copy, Clone, Debug)]
struct EventLoopMessage(Duration);

impl EventLoopMessage {
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }
}

impl EspTypedEventSource for EventLoopMessage {
    fn source() -> *const ffi::c_char {
        b"DEMO-SERVICE\0".as_ptr() as *const _
    }
}

impl EspTypedEventSerializer<EventLoopMessage> for EventLoopMessage {
    fn serialize<R>(
        event: &EventLoopMessage,
        f: impl for<'a> FnOnce(&'a EspEventPostData) -> R,
    ) -> R {
        f(&unsafe { EspEventPostData::new(Self::source(), Self::event_id(), event) })
    }
}

impl EspTypedEventDeserializer<EventLoopMessage> for EventLoopMessage {
    fn deserialize<R>(
        data: &EspEventFetchData,
        f: &mut impl for<'a> FnMut(&'a EventLoopMessage) -> R,
    ) -> R {
        f(unsafe { data.as_payload() })
    }
}

fn test_eventloop() -> Result<(EspBackgroundEventLoop, EspBackgroundSubscription)> {
    use embedded_svc::event_bus::EventBus;

    info!("About to start a background event loop");
    let eventloop = EspBackgroundEventLoop::new(&Default::default())?;

    info!("About to subscribe to the background event loop");
    let subscription = eventloop.subscribe(|message: &EventLoopMessage| {
        info!("Got message from the event loop: {:?}", message.0);
    })?;

    Ok((eventloop, subscription))
}

fn test_mqtt_client() -> Result<EspMqttClient<ConnState<MessageImpl, EspError>>> {
    info!("About to start MQTT client");

    let conf = MqttClientConfiguration {
        client_id: Some("rust-esp32-std-demo"),
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

        ..Default::default()
    };

    let (mut client, mut connection) =
        EspMqttClient::new_with_conn("mqtts://broker.emqx.io:8883", &conf)?;

    info!("MQTT client started");

    // Need to immediately start pumping the connection for messages, or else subscribe() and publish() below will not work
    // Note that when using the alternative constructor - `EspMqttClient::new` - you don't need to
    // spawn a new thread, as the messages will be pumped with a backpressure into the callback you provide.
    // Yet, you still need to efficiently process each message in the callback without blocking for too long.
    //
    // Note also that if you go to http://tools.emqx.io/ and then connect and send a message to topic
    // "rust-esp32-std-demo", the client configured here should receive it.
    thread::spawn(move || {
        info!("MQTT Listening for messages");

        while let Some(msg) = connection.next() {
            match msg {
                Err(e) => info!("MQTT Message ERROR: {}", e),
                Ok(msg) => info!("MQTT Message: {:?}", msg),
            }
        }

        info!("MQTT connection loop exit");
    });

    client.subscribe("rust-esp32-std-demo", QoS::AtMostOnce)?;

    info!("Subscribed to all topics (rust-esp32-std-demo)");

    client.publish(
        "rust-esp32-std-demo",
        QoS::AtMostOnce,
        false,
        "Hello from rust-esp32-std-demo!".as_bytes(),
    )?;

    info!("Published a hello message to topic \"rust-esp32-std-demo\"");

    Ok(client)
}

#[cfg(feature = "experimental")]
mod experimental {
    use core::ffi;

    use log::info;
    use std::{net::TcpListener, net::TcpStream, thread};

    pub fn test() -> anyhow::Result<()> {
        #[cfg(not(esp_idf_version = "4.3"))]
        test_tcp_bind_async()?;

        test_https_client()?;

        Ok(())
    }

    #[cfg(not(esp_idf_version = "4.3"))]
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

        #[allow(clippy::needless_update)]
        {
            esp_idf_sys::esp!(unsafe {
                esp_idf_sys::esp_vfs_eventfd_register(&esp_idf_sys::esp_vfs_eventfd_config_t {
                    max_fds: 5,
                    ..Default::default()
                })
            })?;
        }

        thread::Builder::new().stack_size(4096).spawn(move || {
            smol::block_on(test_tcp_bind()).unwrap();
        })?;

        Ok(())
    }

    fn test_https_client() -> anyhow::Result<()> {
        use embedded_svc::http::{self, client::*, status, Headers, Status};
        use embedded_svc::io::Read;
        use embedded_svc::utils::io;
        use esp_idf_svc::http::client::*;

        let url = String::from("https://google.com");

        info!("About to fetch content from {}", url);

        let mut client = Client::wrap(EspHttpConnection::new(&Configuration {
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

            ..Default::default()
        })?);

        let mut response = client.get(&url)?.submit()?;

        let mut body = [0_u8; 3048];

        let read = io::try_read_full(&mut response, &mut body).map_err(|err| err.0)?;

        info!(
            "Body (truncated to 3K):\n{:?}",
            String::from_utf8_lossy(&body[..read]).into_owned()
        );

        // Complete the response
        while response.read(&mut body)? > 0 {}

        Ok(())
    }
}

#[cfg(feature = "ttgo")]
fn ttgo_hello_world(
    backlight: gpio::Gpio4,
    dc: gpio::Gpio16,
    rst: gpio::Gpio23,
    spi: spi::SPI2,
    sclk: gpio::Gpio18,
    sdo: gpio::Gpio19,
    cs: gpio::Gpio5,
) -> Result<()> {
    info!("About to initialize the TTGO ST7789 LED driver");

    let mut backlight = gpio::PinDriver::output(backlight)?;
    backlight.set_high()?;

    let di = SPIInterfaceNoCS::new(
        spi::SpiDeviceDriver::new_single(
            spi,
            sclk,
            sdo,
            Option::<gpio::Gpio21>::None,
            spi::Dma::Disabled,
            Some(cs),
            &spi::SpiConfig::new().baudrate(26.MHz().into()),
        )?,
        gpio::PinDriver::output(dc)?,
    );

    let mut display = mipidsi::Builder::st7789(di)
        .init(&mut delay::Ets, Some(gpio::PinDriver::output(rst)?))
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    display
        .set_orientation(mipidsi::options::Orientation::Portrait(false))
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    // The TTGO board's screen does not start at offset 0x0, and the physical size is 135x240, instead of 240x320
    let top_left = Point::new(52, 40);
    let size = Size::new(135, 240);

    led_draw(&mut display.cropped(&Rectangle::new(top_left, size)))
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))
}

#[cfg(feature = "kaluga")]
fn kaluga_hello_world(
    backlight: gpio::Gpio6,
    dc: gpio::Gpio13,
    rst: gpio::Gpio16,
    spi: spi::SPI3,
    sclk: gpio::Gpio15,
    sdo: gpio::Gpio9,
    cs: gpio::Gpio11,
) -> Result<()> {
    info!("About to initialize the Kaluga ST7789 SPI LED driver");

    let mut backlight = gpio::PinDriver::output(backlight)?;
    backlight.set_high()?;

    let di = SPIInterfaceNoCS::new(
        spi::SpiDeviceDriver::new_single(
            spi,
            sclk,
            sdo,
            Option::<gpio::AnyIOPin>::None,
            spi::Dma::Disabled,
            Some(cs),
            &spi::SpiConfig::new().baudrate(80.MHz().into()),
        )?,
        gpio::PinDriver::output(dc)?,
    );

    let mut display = mipidsi::Builder::st7789(di)
        .init(&mut delay::Ets, Some(gpio::PinDriver::output(rst)?))
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    display
        .set_orientation(mipidsi::options::Orientation::Landscape(false))
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    led_draw(&mut display).map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    Ok(())
}

#[cfg(feature = "heltec")]
fn heltec_hello_world(
    rst: gpio::Gpio16,
    i2c: i2c::I2C0,
    sda: gpio::Gpio4,
    scl: gpio::Gpio15,
) -> Result<()> {
    info!("About to initialize the Heltec SSD1306 I2C LED driver");

    let di = ssd1306::I2CDisplayInterface::new(i2c::I2cDriver::new(
        i2c,
        sda,
        scl,
        &i2c::I2cConfig::new().baudrate(400.kHz().into()),
    )?);

    let mut delay = delay::Ets;
    let mut reset = gpio::PinDriver::output(rst)?;

    reset.set_high()?;
    delay.delay_ms(1 as u32);

    reset.set_low()?;
    delay.delay_ms(10 as u32);

    reset.set_high()?;

    let mut display = ssd1306::Ssd1306::new(
        di,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display
        .init()
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    led_draw(&mut display).map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    display
        .flush()
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    Ok(())
}

#[cfg(feature = "ssd1306g_spi")]
fn ssd1306g_hello_world_spi(
    dc: gpio::AnyOutputPin,
    rst: gpio::AnyOutputPin,
    spi: impl peripheral::Peripheral<P = impl spi::SpiAnyPins> + 'static,
    sclk: gpio::AnyOutputPin,
    sdo: gpio::AnyOutputPin,
    cs: gpio::AnyOutputPin,
) -> Result<()> {
    info!("About to initialize the SSD1306 SPI LED driver");

    let di = SPIInterfaceNoCS::new(
        spi::SpiDeviceDriver::new_single(
            spi,
            sclk,
            sdo,
            Option::<gpio::AnyIOPin>::None,
            spi::Dma::Disabled,
            Some(cs),
            &spi::SpiConfig::new().baudrate(10.MHz().into()),
        )?,
        gpio::PinDriver::output(dc)?,
    );

    let mut delay = delay::Ets;
    let mut reset = gpio::PinDriver::output(rst)?;

    reset.set_high()?;
    delay.delay_ms(1 as u32);

    reset.set_low()?;
    delay.delay_ms(10 as u32);

    reset.set_high()?;

    let mut display = ssd1306::Ssd1306::new(
        di,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate180,
    )
    .into_buffered_graphics_mode();

    display
        .init()
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    led_draw(&mut display).map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    display
        .flush()
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    Ok(())
}

#[cfg(feature = "ssd1306g")]
fn ssd1306g_hello_world(
    i2c: impl peripheral::Peripheral<P = impl i2c::I2c> + 'static,
    pwr: gpio::AnyOutputPin,
    scl: gpio::AnyIOPin,
    sda: gpio::AnyIOPin,
) -> Result<impl OutputPin<Error = EspError>> {
    info!("About to initialize a generic SSD1306 I2C LED driver");

    let di = ssd1306::I2CDisplayInterface::new(i2c::I2cDriver::new(
        i2c,
        sda,
        scl,
        &i2c::I2cConfig::new().baudrate(400.kHz().into()),
    )?);

    let mut delay = delay::Ets;
    let mut power = gpio::PinDriver::output(pwr)?;

    // Powering an OLED display via an output pin allows one to shutdown the display
    // when it is no longer needed so as to conserve power
    //
    // Of course, the I2C driver should also be properly de-initialized etc.
    power.set_drive_strength(gpio::DriveStrength::I40mA)?;
    power.set_high()?;
    delay.delay_ms(10_u32);

    let mut display = ssd1306::Ssd1306::new(
        di,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display
        .init()
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    led_draw(&mut display).map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    display
        .flush()
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    Ok(power)
}

#[cfg(feature = "esp32s3_usb_otg")]
fn esp32s3_usb_otg_hello_world(
    backlight: gpio::Gpio9,
    dc: gpio::Gpio4,
    rst: gpio::Gpio8,
    spi: spi::SPI3,
    sclk: gpio::Gpio6,
    sdo: gpio::Gpio7,
    cs: gpio::Gpio5,
) -> Result<()> {
    info!("About to initialize the ESP32-S3-USB-OTG SPI LED driver ST7789VW");

    let mut backlight = gpio::PinDriver::output(backlight)?;
    backlight.set_high()?;

    let di = SPIInterfaceNoCS::new(
        spi::SpiDeviceDriver::new_single(
            spi,
            sclk,
            sdo,
            Option::<gpio::AnyIOPin>::None,
            spi::Dma::Disabled,
            Some(cs),
            &spi::SpiConfig::new().baudrate(80.MHz().into()),
        )?,
        gpio::PinDriver::output(dc)?,
    );

    let mut display = mipidsi::Builder::st7789(di)
        .init(&mut delay::Ets, Some(gpio::PinDriver::output(rst)?))
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    display
        .set_orientation(mipidsi::options::Orientation::Landscape(false))
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    led_draw(&mut display).map_err(|e| anyhow::anyhow!("Led draw error: {:?}", e))
}

#[allow(dead_code)]
fn led_draw<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget + Dimensions,
    D::Color: RgbColor,
{
    display.clear(RgbColor::BLACK)?;

    Rectangle::new(display.bounding_box().top_left, display.bounding_box().size)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(RgbColor::BLUE)
                .stroke_color(RgbColor::YELLOW)
                .stroke_width(1)
                .build(),
        )
        .draw(display)?;

    Text::new(
        "Hello Rust!",
        Point::new(10, (display.bounding_box().size.height - 10) as i32 / 2),
        MonoTextStyle::new(&FONT_10X20, RgbColor::WHITE),
    )
    .draw(display)?;

    info!("LED rendering done");

    Ok(())
}

#[allow(unused_variables)]
#[cfg(not(feature = "experimental"))]
fn httpd(mutex: Arc<(Mutex<Option<u32>>, Condvar)>) -> Result<idf::Server> {
    let server = idf::ServerRegistry::new()
        .at("/")
        .get(|_| Ok("Hello from Rust!".into()))?
        .at("/foo")
        .get(|_| bail!("Boo, something happened!"))?
        .at("/bar")
        .get(|_| {
            Response::new(403)
                .status_message("No permissions")
                .body("You have no permissions to access this page".into())
                .into()
        })?
        .at("/panic")
        .get(|_| panic!("User requested a panic!"))?;

    #[cfg(esp32s2)]
    let server = httpd_ulp_endpoints(server, mutex)?;

    server.start(&Default::default())
}

#[cfg(all(esp32s2, not(feature = "experimental")))]
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
                .ok_or(anyhow::anyhow!("No parameter cycles"))??;

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

#[allow(unused_variables)]
#[cfg(feature = "experimental")]
fn httpd(
    mutex: Arc<(Mutex<Option<u32>>, Condvar)>,
) -> Result<esp_idf_svc::http::server::EspHttpServer> {
    use embedded_svc::http::server::{
        Connection, Handler, HandlerResult, Method, Middleware, Query, Request, Response,
    };
    use embedded_svc::io::Write;
    use esp_idf_svc::http::server::{fn_handler, EspHttpConnection, EspHttpServer};

    struct SampleMiddleware {}

    impl<C> Middleware<C> for SampleMiddleware
    where
        C: Connection,
    {
        fn handle<'a, H>(&'a self, connection: &'a mut C, handler: &'a H) -> HandlerResult
        where
            H: Handler<C>,
        {
            let req = Request::wrap(connection);

            info!("Middleware called with uri: {}", req.uri());

            let connection = req.release();

            if let Err(err) = handler.handle(connection) {
                if !connection.is_response_initiated() {
                    let mut resp = Request::wrap(connection).into_status_response(500)?;

                    write!(&mut resp, "ERROR: {err}")?;
                } else {
                    // Nothing can be done as the error happened after the response was initiated, propagate further
                    return Err(err);
                }
            }

            Ok(())
        }
    }

    struct SampleMiddleware2 {}

    impl<C> Middleware<C> for SampleMiddleware2
    where
        C: Connection,
    {
        fn handle<'a, H>(&'a self, connection: &'a mut C, handler: &'a H) -> HandlerResult
        where
            H: Handler<C>,
        {
            info!("Middleware2 called");

            handler.handle(connection)
        }
    }

    let mut server = EspHttpServer::new(&Default::default())?;

    server
        .fn_handler("/", Method::Get, |req| {
            req.into_ok_response()?
                .write_all("Hello from Rust!".as_bytes())?;

            Ok(())
        })?
        .fn_handler("/foo", Method::Get, |_| {
            Result::Err("Boo, something happened!".into())
        })?
        .fn_handler("/bar", Method::Get, |req| {
            req.into_response(403, Some("No permissions"), &[])?
                .write_all("You have no permissions to access this page".as_bytes())?;

            Ok(())
        })?
        .fn_handler("/panic", Method::Get, |_| panic!("User requested a panic!"))?
        .handler(
            "/middleware",
            Method::Get,
            SampleMiddleware {}.compose(fn_handler(|_| {
                Result::Err("Boo, something happened!".into())
            })),
        )?
        .handler(
            "/middleware2",
            Method::Get,
            SampleMiddleware2 {}.compose(SampleMiddleware {}.compose(fn_handler(|req| {
                req.into_ok_response()?
                    .write_all("Middleware2 handler called".as_bytes())?;

                Ok(())
            }))),
        )?;

    #[cfg(esp32s2)]
    httpd_ulp_endpoints(&mut server, mutex)?;

    Ok(server)
}

#[cfg(all(esp32s2, feature = "experimental"))]
fn httpd_ulp_endpoints(
    server: &mut esp_idf_svc::http::server::EspHttpServer,
    mutex: Arc<(Mutex<Option<u32>>, Condvar)>,
) -> Result<()> {
    use embedded_svc::http::server::registry::Registry;
    use embedded_svc::http::server::{Request, Response};
    use embedded_svc::io::adapters::ToStd;

    server
        .handle_get("/ulp", |_req, resp| {
            resp.send_str(
            r#"
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
            "#)?;

            Ok(())
        })?
        .handle_post("/ulp_start", move |mut req, resp| {
            let mut body = Vec::new();

            ToStd::new(req.reader()).read_to_end(&mut body)?;

            let cycles = url::form_urlencoded::parse(&body)
                .filter(|p| p.0 == "cycles")
                .map(|p| str::parse::<u32>(&p.1).map_err(Error::msg))
                .next()
                .ok_or(anyhow::anyhow!("No parameter cycles"))??;

            let mut wait = mutex.0.lock().unwrap();

            *wait = Some(cycles);
            mutex.1.notify_one();

            resp.send_str(
                &format!(
                r#"
                <doctype html5>
                <html>
                    <body>
                        About to sleep now. The ULP chip should blink the LED {} times and then wake me up. Bye!
                    </body>
                </html>
                "#,
                cycles))?;

            Ok(())
        })?;

    Ok(())
}

#[cfg(esp32s2)]
fn start_ulp(mut ulp: esp_idf_hal::ulp::ULP, cycles: u32) -> Result<()> {
    let cycles_var = CYCLES as *mut u32;

    unsafe {
        ulp.load(ULP)?;
        info!("RiscV ULP binary loaded successfully");

        info!(
            "Default ULP LED blink cycles: {}",
            ulp.read_var(cycles_var)?
        );

        ulp.write_var(cycles_var, cycles)?;
        info!(
            "Sent {} LED blink cycles to the ULP",
            ulp.read_var(cycles_var)?
        );

        ulp.start()?;
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
#[allow(dead_code)]
fn wifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    use std::net::Ipv4Addr;

    use esp_idf_svc::handle::RawHandle;

    let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), None)?);

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

    wifi.start()?;

    info!("Starting wifi...");

    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        bail!("Wifi did not start");
    }

    info!("Connecting wifi...");

    wifi.connect()?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(
        Duration::from_secs(20),
        || {
            wifi.is_connected().unwrap()
                && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        },
    ) {
        bail!("Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    ping(ip_info.subnet.gateway)?;

    Ok(wifi)
}

#[cfg(any(feature = "qemu", feature = "w5500", feature = "ip101"))]
fn eth_configure(
    sysloop: &EspSystemEventLoop,
    mut eth: Box<esp_idf_svc::eth::EspEth<'static>>,
) -> Result<Box<esp_idf_svc::eth::EspEth<'static>>> {
    use std::net::Ipv4Addr;

    info!("Eth created");

    eth.start()?;

    info!("Starting eth...");

    if !esp_idf_svc::eth::EthWait::new(eth.driver(), sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || eth.is_started().unwrap())
    {
        bail!("Eth did not start");
    }

    if !EspNetifWait::new::<EspNetif>(eth.netif(), &sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || {
            eth.netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        })
    {
        bail!("Eth did not receive a DHCP lease");
    }

    let ip_info = eth.netif().get_ip_info()?;

    info!("Eth DHCP info: {:?}", ip_info);

    ping(ip_info.subnet.gateway)?;

    Ok(eth)
}

fn ping(ip: ipv4::Ipv4Addr) -> Result<()> {
    info!("About to do some pings for {:?}", ip);

    let ping_summary = ping::EspPing::default().ping(ip, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!("Pinging IP {} resulted in timeouts", ip);
    }

    info!("Pinging done");

    Ok(())
}

#[cfg(not(feature = "qemu"))]
#[cfg(esp_idf_lwip_ipv4_napt)]
fn enable_napt(wifi: &mut EspWifi) -> Result<()> {
    wifi.ap_netif_mut().enable_napt(true);

    info!("NAPT enabled on the WiFi SoftAP!");

    Ok(())
}

#[cfg(feature = "waveshare_epd")]
fn waveshare_epd_hello_world(
    spi: impl peripheral::Peripheral<P = impl spi::SpiAnyPins> + 'static,
    sclk: gpio::AnyOutputPin,
    sdo: gpio::AnyOutputPin,
    cs: gpio::AnyOutputPin,
    busy_in: gpio::AnyInputPin,
    dc: gpio::AnyOutputPin,
    rst: gpio::AnyOutputPin,
) -> Result<()> {
    info!("About to initialize Waveshare 4.2 e-paper display");

    let mut driver = spi::SpiDeviceDriver::new_single(
        spi,
        sclk,
        sdo,
        Option::<gpio::AnyIOPin>::None,
        spi::Dma::Disabled,
        Option::<gpio::AnyOutputPin>::None,
        &spi::SpiConfig::new().baudrate(26.MHz().into()),
    )?;

    // Setup EPD
    let mut epd = Epd4in2::new(
        &mut driver,
        gpio::PinDriver::output(cs)?,
        gpio::PinDriver::input(busy_in)?,
        gpio::PinDriver::output(dc)?,
        gpio::PinDriver::output(rst)?,
        &mut delay::Ets,
    )
    .unwrap();

    // Use display graphics from embedded-graphics
    let mut buffer =
        vec![DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH as usize / 8 * HEIGHT as usize];
    let mut display = VarDisplay::new(WIDTH, HEIGHT, &mut buffer);

    let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

    // Create a text at position (20, 30) and draw it using the previously defined style
    Text::new("Hello Rust!", Point::new(20, 30), style).draw(&mut display)?;

    // Display updated frame
    epd.update_frame(&mut driver, &display.buffer(), &mut delay::Ets)?;
    epd.display_frame(&mut driver, &mut delay::Ets)?;

    Ok(())
}
