fn main() {
    let dst = cmake::Config::new(".")
        .define("BUILD_SHARED_LIBS", "OFF").build();

    let lib_dir = dst.join("build/lib");
    println!("cargo:rustc-link-search={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=cd");

    pkg_config::Config::new().probe("libcdio_cdda").expect("libcdio_cdda not found");
    pkg_config::Config::new().probe("libcdio").expect("libcdio library not found");

    println!("cargo:rustc-link-lib=cdio_cdda");
    println!("cargo:rustc-link-lib=cdda_interface");

    println!("cargo:rerun-if-changed=c_src/cd.c");
    println!("cargo:rerun-if-changed=c_src/cd.h");
}
