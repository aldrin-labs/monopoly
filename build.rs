use std::env;

fn main() {
    // Force using system toolchain instead of Solana's bundled one
    env::set_var("RUSTC_WRAPPER", "");
    env::set_var("RUSTFLAGS", "--cfg force_system_toolchain");
    
    // Ensure we're using the project's toolchain
    println!("cargo:rustc-env=RUST_TOOLCHAIN=1.75.0");
    println!("cargo:rerun-if-changed=rust-toolchain.toml");
}
