// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> anyhow::Result<()> {
    pio::SconsVariables::output_propagated_cargo_link_args("ESP_IDF")
}
