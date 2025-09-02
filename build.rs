fn main() {
    println!("cargo:rustc-link-lib=quirc");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
}
