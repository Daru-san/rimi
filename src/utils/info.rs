use super::color::ColorInfo;
use image::{DynamicImage, ImageFormat};
use std::fs::metadata;
use std::path::PathBuf;

// TODO: Pretty displaying
pub fn print_info(image: &DynamicImage, path: PathBuf, do_short: bool) {
    let (height, width) = (image.height(), image.width());
    let format = ImageFormat::from_path(&path).expect("Error decoding image format: ");

    let meta = metadata(&path).unwrap();
    let size = meta.len();

    let color_info = ColorInfo::from_image(image);

    println!("Image file: {:?}", path.as_os_str());
    println!("File size: {} bytes", size);
    println!("Dimensions: {}x{}", width, height);
    println!("Format: {}", format.to_mime_type());

    if !do_short {
        println!("Color space: {}", color_info.color_space);
        println!("Bit depth: {}", color_info.bit_depth);
    }
}
