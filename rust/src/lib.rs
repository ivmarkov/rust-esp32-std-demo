#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use std::time::*;
use std::thread;

mod wip;

#[no_mangle]
pub extern "C" fn rust_main() {
    simple_playground();

    threads_playground();

    // Enough playing. Ignite the Rocket framework
    println!("Igniting Rocket...");
    thread::spawn(move || {
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
