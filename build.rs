fn main() {
    // Describe how to build C files for tests
    cc::Build::new().file("tests/gen.c").compile("gen");
}
