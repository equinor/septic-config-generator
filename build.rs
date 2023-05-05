#[cfg(windows)]
fn main() {
    use winresource::WindowsResource;

    let mut res = WindowsResource::new();
    res.set_language(0x0009);
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {}
