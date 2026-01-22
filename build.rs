use std::{fs::{self, File}};

use ico::{IconDir, IconDirEntry, IconImage};

fn main() {
    // println!("cargo:rustc-link-lib=dylib=stdc++");
    // println!("cargo:rustc-link-lib=static=pdfium");
    // println!("cargo:rustc-link-search=native=./");
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let image = image::open("icon.png").expect("Error building application");
        let resized_image = image.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
        let rgba_image = resized_image.to_rgba8();
        let icon_image = IconImage::from_rgba_data(256, 256, rgba_image.to_vec());
        let mut icon_dir = IconDir::new(ico::ResourceType::Icon);
        icon_dir.add_entry(IconDirEntry::encode(&icon_image).expect("Error building application"));
        let icon_file = File::create("icon.ico").expect("Error building application");
        icon_dir.write(icon_file).expect("Error building application");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile().expect("Error building application");
        fs::remove_file("icon.ico").expect("Error building application");
    }
}