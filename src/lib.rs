#![feature(unboxed_closures)]
#![feature(proc_macro_hygiene, decl_macro)]
//#[macro_use] extern crate rocket;

use std::{env, sync::Arc, time::*};
use std::thread;

use esp_idf_svc::nvs::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::ping;
use esp_idf_svc::wifi::*;

use embedded_svc::ping::Ping;
use embedded_svc::wifi::*;
use embedded_svc::httpd::*;
use embedded_svc::httpd::registry::*;

use esp_idf_svc::httpd as idf;
//use esp_idf_svc::httpd::Registry;

use anyhow::*;
use log::*;

#[no_mangle]
fn main() -> Result<()> {
    simple_playground();

    threads_playground();

    // Enough playing.
    // The real demo: start WiFi and ignite Httpd

    env::set_var("RUST_BACKTRACE", "1"); // Get some nice backtraces from Anyhow

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

    // Check calling into C (more of a Cargo <-> PlatfromIO hybrid project demo than anything else)

    extern "C" {
        fn get_magic_number() -> i32;
    }

    let magic_number = unsafe {get_magic_number()};
    println!("Calling into C to get The Answer: {}. Big surprise.", magic_number);
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

// fn rocket() {
//     println!("Igniting Rocket...");
//     thread::spawn(move || { // In a separate thread for now, because the main thread stack is only 3K, which is not enough
//         #[get("/")]
//         fn index() -> &'static str {
//             "Hello, world!"
//         }

//         rocket::ignite().mount("/", routes![index]).launch();
//     })
//     .join()
//     .unwrap();
// }

fn wifi() -> Result<EspWifi> {
    let mut wifi = EspWifi::new(
        Arc::new(EspNetif::new()?),
        Arc::new(EspSysLoop::new()?),
        Arc::new(EspDefaultNvs::new()?))?;

    info!("Wifi created");

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: "ssid".into(),
        password: "password".into(),
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
