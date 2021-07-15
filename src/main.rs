use std::thread;
use std::{env, sync::Arc, time::*};

use anyhow::*;
use log::*;

use embedded_svc::anyerror::*;
use embedded_svc::httpd::registry::*;
use embedded_svc::httpd::*;
use embedded_svc::ping::Ping;
use embedded_svc::wifi::*;

use esp_idf_svc::httpd as idf;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;

use esp_idf_hal::delay;
use esp_idf_hal::gpio;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi;

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;

use ili9341;
use st7789;

fn main() -> Result<()> {
    simple_playground();

    threads_playground();

    // Enough playing.
    // The real demo: start WiFi and ignite Httpd

    env::set_var("RUST_BACKTRACE", "1"); // Get some nice backtraces from Anyhow

    // Uncomment this if you have a TTGO ESP32 board
    // For other boards, you might have to use a different embedded-graphics driver and pin configuration
    ttgo_hello_world()?;

    // ... or uncomment this if you have a Kaluga-1 ESP32-S2 board
    // For other boards, you might have to use a different embedded-graphics driver and pin configuration
    // kaluga_hello_world(true)?;

    let _wifi = wifi()?;

    let _httpd = httpd()?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

fn simple_playground() {
    // Start simple
    println!("Hello, world from Rust!");

    // Check collections
    let mut children = vec![];

    children.push("foo");
    children.push("bar");
    println!("More complex print {:?}", children);
}

fn threads_playground() {
    let mut children = vec![];

    println!("Rust main thread: {:?}", thread::current());

    for i in 0..5 {
        // Spin up another thread
        children.push(thread::spawn(move || {
            println!("This is thread number {}, {:?}", i, thread::current());
        }));
    }

    println!(
        "About to join the threads. If ESP-IDF was patched successfully, joining will NOT crash"
    );

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }

    thread::sleep(Duration::new(2, 0));

    println!("Joins were successful.");
}

#[allow(dead_code)]
fn ttgo_hello_world() -> Result<()> {
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
        // SP7789V is for as 240x320 device, even though the screen is smaller
        240,
        320,
    );

    AnyError::<st7789::Error<_>>::wrap(|| {
        display.init(&mut delay::Ets)?;
        display.set_orientation(st7789::Orientation::Portrait)?;

        // The TTGO board's screen does not start at offset 0x0, and the physical size is 135x240, instead of 240x320
        let top_left = Point::new(52, 40);
        let size = Size::new(135, 240);

        //led_draw(&mut display)
        led_draw(&mut display.cropped(&Rectangle::new(top_left, size)))
    })
}

#[allow(dead_code)]
fn kaluga_hello_world(ili9341: bool) -> Result<()> {
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
                .stroke_color(RgbColor::RED)
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

fn httpd() -> Result<idf::Server> {
    idf::ServerRegistry::new()
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
        })?
        .start(&Default::default())
}

fn wifi() -> Result<impl Wifi> {
    let mut wifi = EspWifi::new(
        Arc::new(EspNetif::new()?),
        Arc::new(EspSysLoop::new()?),
        Arc::new(EspDefaultNvs::new()?),
    )?;

    info!("Wifi created");

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: "ssid".into(),
        password: "pass".into(),
        ..Default::default()
    }))?;

    info!("Wifi configuration set, about to get status");

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        _,
    ) = status
    {
        info!("Wifi connected, about to do some pings");

        let ping_summary =
            ping::EspPing::default().ping(ip_settings.subnet.gateway, &Default::default())?;
        if ping_summary.transmitted != ping_summary.received {
            bail!(
                "Pinging gateway {} resulted in timeouts",
                ip_settings.subnet.gateway
            );
        }

        info!("Pinging done");
    } else {
        bail!("Unexpected Wifi status: {:?}", &status);
    }

    Ok(wifi)
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
        match self {
            Self::Landscape | Self::LandscapeFlipped => true,
            Self::Portrait | Self::PortraitFlipped => false,
        }
    }
}
