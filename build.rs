use std::path::PathBuf;

use embuild::{self, bingen, cargo, symgen};

fn main() {
    embuild::espidf::sysenv::output();

    let cfg = embuild::espidf::sysenv::cfg_args().unwrap();
    if cfg.get("esp32s2").is_some() {
        // Future; might be possible once https://github.com/rust-lang/cargo/issues/9096 hits Cargo nightly:
        //let ulp_elf = PathBuf::from(env::var_os("CARGO_BIN_FILE_RUST_ESP32_ULP_BLINK_rust_esp32_ulp_blink").unwrap());

        let ulp_elf = PathBuf::from("ulp").join("rust-esp32-ulp-blink");
        cargo::track_file(&ulp_elf);

        // This is where the RTC Slow Mem is mapped within the ESP32-S2 memory space
        let ulp_bin = symgen::Symgen::new(&ulp_elf, 0x5000_0000_u64)
            .run()
            .unwrap();
        cargo::track_file(ulp_bin);

        let ulp_sym = bingen::Bingen::new(ulp_elf).run().unwrap();
        cargo::track_file(ulp_sym);
    }
}
