#![feature(unboxed_closures)]
#![feature(proc_macro_hygiene, decl_macro)]

use std::{env, sync::Arc, time::*};
use std::thread;

use embedded_graphics::image::Image;
use esp_idf_svc::nvs::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::ping;
use esp_idf_svc::wifi::*;

use embedded_svc::ping::Ping;
use embedded_svc::wifi::*;
use embedded_svc::httpd::*;
use embedded_svc::httpd::registry::*;
use embedded_svc::anyerror::*;

use esp_idf_svc::httpd as idf;
//use esp_idf_svc::httpd::Registry;

use esp_idf_hal::prelude::*;
use esp_idf_hal::delay;
use esp_idf_hal::gpio;
use esp_idf_hal::spi;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::*;
use embedded_graphics::primitives::*;
use embedded_graphics::style::*;
use embedded_graphics::fonts::{Font12x16, Text};

use anyhow::*;
use log::*;
use st7789::*;

pub struct SubDrawTarget<D> {
    target: D,
    bounds: Rectangle,
}

impl<D> SubDrawTarget<D> {
    pub fn new(target: D, bounds: Rectangle) -> Self {
        Self {
            target,
            bounds,
        }
    }

    #[inline(always)]
    fn translate_pixel<C: PixelColor>(&self, pixel: &Pixel<C>) -> Pixel<C> {
        Pixel(pixel.0 + self.bounds.top_left, pixel.1)
    }

    #[inline(always)]
    fn translate<T: Transform>(&self, transformable: &T) -> T {
        transformable.translate(self.bounds.top_left)
    }
}

impl<C: PixelColor, D: DrawTarget<C>> DrawTarget<C> for SubDrawTarget<D> {
    type Error = D::Error;

    #[inline(always)]
    fn draw_pixel(&mut self, item: Pixel<C>) -> Result<(), Self::Error> {
        self.target.draw_pixel(self.translate_pixel(&item))
    }

    #[inline(always)]
    fn draw_iter<T>(&mut self, item: T) -> Result<(), Self::Error>
    where
        T: IntoIterator<Item = Pixel<C>>,
    {
        for pixel in item {
            self.draw_pixel(pixel)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn size(&self) -> Size {
        self.bounds.size()
    }

    #[inline(always)]
    fn clear(&mut self, color: C) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        self.target.clear(color)
    }

    #[inline(always)]
    fn draw_line(
        &mut self,
        item: &Styled<Line, PrimitiveStyle<C>>,
    ) -> Result<(), Self::Error> {
        self.target.draw_line(&self.translate(item))
    }

    #[inline(always)]
    fn draw_triangle(
        &mut self,
        item: &Styled<Triangle, PrimitiveStyle<C>>,
    ) -> Result<(), Self::Error> {
        self.target.draw_triangle(&self.translate(item))
    }

    #[inline(always)]
    fn draw_rectangle(
        &mut self,
        item: &Styled<Rectangle, PrimitiveStyle<C>>,
    ) -> Result<(), Self::Error> {
        self.target.draw_rectangle(&self.translate(item))
    }

    #[inline(always)]
    fn draw_circle(
        &mut self,
        item: &Styled<Circle, PrimitiveStyle<C>>,
    ) -> Result<(), Self::Error> {
        self.target.draw_circle(&self.translate(item))
    }

    #[inline(always)]
    fn draw_image<'a, 'b, I>(&mut self, item: &'a Image<'b, I, C>) -> Result<(), Self::Error>
    where
        &'b I: IntoPixelIter<C>,
        I: ImageDimensions,
        C: PixelColor + From<<C as PixelColor>::Raw>,
    {
        self.target.draw_image(&self.translate(item))
    }
}

fn main() -> Result<()> {
    simple_playground();

    threads_playground();

    // Enough playing.
    // The real demo: start WiFi and ignite Httpd

    env::set_var("RUST_BACKTRACE", "1"); // Get some nice backtraces from Anyhow

    // Uncomment this if you have a TTGO board
    // For other boards, you might have to use a different embedded-graphics driver and pin configuration
    // let _gfx = gfx_hello_world()?;

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

    println!("About to join the threads. If ESP-IDF was patched successfully, joining will NOT crash");

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }

    thread::sleep(Duration::new(2, 0));

    println!("Joins were successful.");
}

fn gfx_hello_world() -> Result<
        SubDrawTarget<ST7789<SPIInterfaceNoCS<spi::Master<
            spi::SPI2,
            gpio::Gpio18<gpio::Unknown>,
            gpio::Gpio19<gpio::Unknown>,
            gpio::Gpio21<gpio::Unknown>,
            gpio::Gpio5<gpio::Unknown>>,
            gpio::Gpio16<gpio::Output>>,
            gpio::Gpio23<gpio::Output>>>> {
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let mut backlight = pins.gpio4.into_output()?;
    backlight.set_high()?;

    let config = <spi::config::Config as Default>::default()
        .baudrate(26.MHz().into())
        .bit_order(spi::config::BitOrder::MSBFirst);

    let spi_master = spi::Master::<spi::SPI2, _, _, _, _>::new(
        peripherals.spi2,
        spi::Pins {
            sclk: pins.gpio18,
            sdo: pins.gpio19,
            sdi: Option::<gpio::Gpio21<gpio::Unknown>>::None,
            cs: Some(pins.gpio5),
        },
        config)?;

    let di = SPIInterfaceNoCS::new(
        spi_master,
        pins.gpio16.into_output()?);

    let top_left = Point::new(53, 40);
    let size = Size::new(135, 240);

    let bounds = Rectangle::new(top_left, top_left + size);

    // create driver
    let mut display = ST7789::new(
        di,
        pins.gpio23.into_output()?,
        bounds.size().width as u16,
        bounds.size().width as u16);

    AnyError::<st7789::Error<_>>::wrap(|| {
        // initialize
        display.init(&mut delay::Ets)?;

        display.clear(Rgb565::BLACK)?;

        // set default orientation
        display.set_orientation(Orientation::Landscape)?;

        let mut display = SubDrawTarget::new(
            display,
            Rectangle::new(
                Point::new(bounds.top_left.y, bounds.top_left.x),
                Point::new(bounds.bottom_right.y, bounds.bottom_right.x)));

        Rectangle::new(Point::zero(), Point::zero() + display.size() - Size::new(1, 1))
            .into_styled(PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLUE)
                .stroke_color(Rgb565::RED)
                .stroke_width(1) // > 1 is not currently supported in embedded-graphics on triangles
                .build())
            .draw(&mut display)?;

        Text::new("Hello Rust!", Point::new(20, (display.size().height - 16) as i32 / 2))
            .into_styled(TextStyle::new(Font12x16, Rgb565::WHITE))
            .draw(&mut display)?;

        info!("LED rendering done");

        Ok(display)
    })
}

fn httpd() -> Result<idf::Server> {
    idf::ServerRegistry::new()
        .at("/").get(|_| Ok("Hello, world!".into()))?
        .at("/foo").get(|_| bail!("Boo, something happened!"))?
        .at("/bar").get(|_| Response::new(403)
            .status_message("No permissions")
            .body("You have no permissions to access this page".into())
            .into())?
        .start(&Default::default())
}

fn wifi() -> Result<EspWifi> {
    let mut wifi = EspWifi::new(
        Arc::new(EspNetif::new()?),
        Arc::new(EspSysLoop::new()?),
        Arc::new(EspDefaultNvs::new()?))?;

    info!("Wifi created");

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: "user".into(),
        password: "pass".into(),
        ..Default::default()
    }))?;

    info!("Wifi configuration set, about to get status");

    let status = wifi.get_status();

    if let Status(ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))), _) = status {
        info!("Wifi connected, about to do some pings");

        let ping_summary = ping::EspPing.ping_summary(ip_settings.subnet.gateway, &Default::default())?;
        if ping_summary.transmitted != ping_summary.received {
            bail!("Pinging gateway {} resulted in timeouts", ip_settings.subnet.gateway);
        }

        info!("Pinging done");
    } else {
        bail!("Unexpected Wifi status: {:?}", &status);
    }

    Ok(wifi)
}
