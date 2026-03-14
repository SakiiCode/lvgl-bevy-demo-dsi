fn main() {
    embuild::espidf::sysenv::output();
    // these env variables must be set based on your installation
    // put them into .cargo/config-local.toml
    let _bindgen_extra_clang_args = env!("BINDGEN_EXTRA_CLANG_ARGS");
}
