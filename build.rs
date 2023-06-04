fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows"
        && std::env::var("PROFILE").unwrap() == "release"
    {
        use winresource::WindowsResource;

        let mut res = WindowsResource::new();

        res.set_language(0x0009); // English
        res.compile().unwrap();
    }
}
