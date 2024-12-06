use std::fs::metadata;
use std::path::PathBuf;

use crate::utils::format_from_path;

use image::{ColorType, DynamicImage};

struct ColorInfo {
    bits: u32,
    color_type: String,
    is_alpha: bool,
}

pub fn print_info(image: &DynamicImage, path: PathBuf, do_short: bool) {
    let (height, width) = (image.height(), image.width());
    let format = ImageFormat::from_path(&path).expect("Error decoding image format: ");

    let meta = metadata(&path).unwrap();
    let size = meta.len();

    println!("Image file: {:?}", path.as_os_str());
    println!("File size: {} bytes", size);
    println!("Dimensions: {}x{}", width, height);
    println!("Format: {}", format.to_mime_type());

    let color_info = match image.color() {
        ColorType::L8 => ColorInfo {
            bits: 8,
            color_type: "Luminant".to_string(),
            is_alpha: false,
        },
        ColorType::La8 => ColorInfo {
            bits: 8,
            color_type: "Luminant".to_string(),
            is_alpha: true,
        },
        ColorType::L16 => ColorInfo {
            bits: 16,
            color_type: "Luminant".to_string(),
            is_alpha: true,
        },
        ColorType::La16 => ColorInfo {
            bits: 16,
            color_type: "Luminant".to_string(),
            is_alpha: true,
        },
        ColorType::Rgb8 => ColorInfo {
            bits: 8,
            color_type: "RGB".to_string(),
            is_alpha: false,
        },
        ColorType::Rgba8 => ColorInfo {
            bits: 8,
            color_type: "RGB".to_string(),
            is_alpha: true,
        },
        ColorType::Rgb16 => ColorInfo {
            bits: 16,
            color_type: "RGB".to_string(),
            is_alpha: false,
        },
        ColorType::Rgba16 => ColorInfo {
            bits: 16,
            color_type: "RGB".to_string(),
            is_alpha: true,
        },
        ColorType::Rgb32F => ColorInfo {
            bits: 32,
            color_type: "RGB".to_string(),
            is_alpha: false,
        },
        ColorType::Rgba32F => ColorInfo {
            bits: 32,
            color_type: "RGB".to_string(),
            is_alpha: true,
        },
        _ => ColorInfo {
            bits: 8,
            color_type: "Unknown".to_string(),
            is_alpha: false,
        },
    };
    if !do_short {
        println!("Color space: {}", color_info.color_type);
        println!("Bit depth: {}", color_info.bits);
        println!("Alpha: {}", color_info.is_alpha);
    }
}
