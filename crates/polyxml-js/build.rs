fn main() {
    let _ = std::panic::catch_unwind(napi_build::setup);
}
