fn main() {
    //build the builder lib
    cc::Build::new()
        .static_flag(true)
        .opt_level_str("O3")
        .file("builder.cpp")
        .debug(false)
        .compile("builder");
}
