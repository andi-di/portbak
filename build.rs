fn main() {
    println!("cargo:rerun-if-changed=assets/icon/portbak.ico");
    println!("cargo:rerun-if-env-changed=PORTBAK_RELEASE_VERSION");
    let version = std::env::var("PORTBAK_RELEASE_VERSION")
        .unwrap_or_else(|_| std::env::var("CARGO_PKG_VERSION").unwrap());
    let parts: Vec<u16> = version
        .split('.')
        .map(|part| {
            part.parse()
                .expect("version components must fit Windows version metadata")
        })
        .collect();
    assert_eq!(parts.len(), 3, "release version must be major.minor.patch");
    println!("cargo:rustc-env=PORTBAK_VERSION={version}");

    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let numeric_version =
            (u64::from(parts[0]) << 48) | (u64::from(parts[1]) << 32) | (u64::from(parts[2]) << 16);
        winresource::WindowsResource::new()
            .set_icon("assets/icon/portbak.ico")
            .set("FileVersion", &version)
            .set("ProductVersion", &version)
            .set_version_info(winresource::VersionInfo::FILEVERSION, numeric_version)
            .set_version_info(winresource::VersionInfo::PRODUCTVERSION, numeric_version)
            .compile()
            .expect("could not embed the Windows executable icon");
    }
}
