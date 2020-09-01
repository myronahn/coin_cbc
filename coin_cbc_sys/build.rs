fn main() {
    println!("cargo:rustc-link-lib=dylib=Cbc");
    println!("cargo:rustc-link-lib=dylib=CoinUtils");
    println!("cargo:rustc-link-lib=dylib=OsiClp");
}
