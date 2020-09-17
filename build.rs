// MIT/Apache2 License

use std::env;

fn main() -> Result<(), &'static str> {
    let target = env::var("TARGET").unwrap();
    if target.contains("windows") {
        cc::Build::new()
            .cpp(true)
            .file("external/win32_gdiplus/cgdiplus.cc")
            .compile("libcgdiplus.a");
        println!("cargo:rustc-link-lib=cgdiplus");
        println!("cargo:rerun-if-changed=external/win32_gdiplus/cgdiplus.h");
        println!("cargo:rerun-if-changed=external/win32_gdiplus/cgdiplus.cc");
        println!("cargo:rerun-if-changed=src/backend/win32/gdi/external.rs");
    }

    Ok(())
}
