use std::path::PathBuf;

use image::{ColorType, DynamicImage};

struct ColorInfo {
    bits: u32,
    color_type: String,
    is_alpha: bool,
}

pub fn print_info(image: &DynamicImage, path: PathBuf, do_short: bool) {
    let (height, width) = (image.height(), image.width());

    println!("Image file: {:?}", path.as_os_str());
    println!("Dimensions: {}x{}", width, height);

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
        println!("Color type: {}", color_info.color_type);
        println!("Bits: {}", color_info.bits);
        println!("Alpha: {}", color_info.is_alpha);
    }
}
