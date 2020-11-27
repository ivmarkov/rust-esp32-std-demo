#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::time::*;
use std::thread;

use std::ffi::{CString};
use esp_idf_sys::*;

mod wip;

#[no_mangle]
pub extern "C" fn main() {
    simple_playground();

    threads_playground();

    // Enough playing. Start WiFi and ignite the Rocket framework

    if let Err(error) = init_peripherals() {
        error.panic();
    }

    println!("Igniting Rocket...");
    thread::spawn(move || { // In a separate thread for now, because the main thread stack is only 3K, which is not enough
        #[get("/")]
        fn index() -> &'static str {
            "Hello, world!"
        }
        
        rocket::ignite().mount("/", routes![index]).launch();
    }).join().unwrap();
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

// NOTE:
// This is currently one big unsafe blob, relying on the C ESP-IDF API (exposed via the esp-idf-sys crate).
//
// Next steps should be in the direction of reducing the need to step down to the unsafe and raw esp-idf-sys API for configuring hardware:
// 1. Extend the existing ESP WiFi driver (https://github.com/esp-rs/esp32-wifi) with an optional feature that drives it on top of 
//    ESP-IDF rather than on bare metal (or write a new, similar driver)
// 2. Bring back to life the rusty embedded-hal-on-top-of-ESP-IDF work (https://github.com/sapir/esp-idf-hal), 
//    possibly by looking at its bare metal equivalent (https://github.com/esp-rs/esp32-hal)
// 3. Think how to write wrappers for stuff which does not have bare metal traits & implementation in esp-rs. 
//    E.g., various drivers, but most importantly - the ESP event loop system and the netif layer.
fn init_peripherals() -> Result<(), Error> {
    unsafe {
        esp!(esp_netif_init())?;
        esp!(esp_event_loop_create_default())?;

        if let Some(err) = Error::from(nvs_flash_init()) {
            match err.code() as u32 {
                ESP_ERR_NVS_NO_FREE_PAGES | ESP_ERR_NVS_NEW_VERSION_FOUND => esp!(nvs_flash_erase())?,
                _ => ()
            }
        }

        esp!(nvs_flash_init())?;

        let cfg = wifi_init_config_t {
            event_handler: Some(esp_event_send_internal),
            osi_funcs: &mut g_wifi_osi_funcs,
            wpa_crypto_funcs: g_wifi_default_wpa_crypto_funcs,
            static_rx_buf_num: 10,
            dynamic_rx_buf_num: 32,
            tx_buf_type: 1,
            static_tx_buf_num: 0,
            dynamic_tx_buf_num: 32,
            csi_enable: 0,
            ampdu_rx_enable: 1,
            ampdu_tx_enable: 1,
            nvs_enable: 0,
            nano_enable: 0,
            tx_ba_win: 6,
            rx_ba_win: 6,
            wifi_task_core_id: 0,
            beacon_max_len: 752,
            mgmt_sbuf_num: 32,
            feature_caps: 1, // CONFIG_FEATURE_WPA3_SAE_BIT
            magic: 0x1F2F3F4F,
        };
        esp!(esp_wifi_init(&cfg))?;
    
        esp!(esp_event_handler_register(WIFI_EVENT, ESP_EVENT_ANY_ID, Option::Some(event_handler), std::ptr::null_mut()))?;
        esp!(esp_event_handler_register(IP_EVENT, ip_event_t_IP_EVENT_STA_GOT_IP as i32, Option::Some(event_handler), std::ptr::null_mut()))?;
    
        // Initialize default station as network interface instance (esp-netif)
        let _esp_netif_t = esp_netif_create_default_wifi_sta();
    
        // Initialize and start WiFi
        let mut wifi_config = wifi_config_t {
            sta: wifi_sta_config_t {
                ssid: [0; 32],
                password: [0; 64],
                scan_method: wifi_scan_method_t_WIFI_FAST_SCAN,
                bssid_set: false,
                bssid: [0; 6],
                channel: 0,
                listen_interval: 0,
                sort_method: wifi_sort_method_t_WIFI_CONNECT_AP_BY_SIGNAL,
                threshold: wifi_scan_threshold_t {rssi: 127, authmode: wifi_auth_mode_t_WIFI_AUTH_OPEN},
                pmf_cfg:  wifi_pmf_config_t {capable: false, required: false},
            }
        };

        set_str(&mut wifi_config.sta.ssid, "ssid");
        set_str(&mut wifi_config.sta.password, "pass");

        esp!(esp_wifi_set_mode(wifi_mode_t_WIFI_MODE_STA))?;
        esp!(esp_wifi_set_config(esp_interface_t_ESP_IF_WIFI_STA, &mut wifi_config))?;
        esp!(esp_wifi_start())
    }
}

unsafe extern "C" fn event_handler(_arg: *mut c_types::c_void, event_base: esp_event_base_t, event_id: c_types::c_int, event_data: *mut c_types::c_void) {
    if event_base == WIFI_EVENT && event_id == wifi_event_t_WIFI_EVENT_STA_START as i32 {
        esp_nofail!(esp_wifi_connect());
    } else if event_base == WIFI_EVENT && event_id == wifi_event_t_WIFI_EVENT_STA_DISCONNECTED as i32 {
        esp_nofail!(esp_wifi_connect());
    } else if event_base == IP_EVENT && event_id == ip_event_t_IP_EVENT_STA_GOT_IP as i32 {
        let event: *const ip_event_got_ip_t = std::mem::transmute(event_data);
        println!("NETIF: Got IP: {:?}", (*event).ip_info);
    }
}

fn set_str(buf: &mut [u8], s: &str) {
    let cs = CString::new(s).unwrap();
    let ss: &[u8] = cs.as_bytes_with_nul();
    buf[..ss.len()].copy_from_slice(&ss);
}
