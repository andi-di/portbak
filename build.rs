fn main() {
    println!("cargo:rerun-if-changed=assets/icon/portbak.ico");

    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/icon/portbak.ico")
            .compile()
            .expect("could not embed the Windows executable icon");
    }
}
