// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> anyhow::Result<()> {
    println!("cargo:rustc-cfg={}", std::env::var("DEP_ESP_IDF_MCU").unwrap());
    pio::cargo::build::LinkArgs::output_propagated("ESP_IDF")
}
