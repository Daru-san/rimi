use dialoguer::Confirm;
use image::imageops::FilterType;
use image::{self, ColorType, DynamicImage, ImageFormat};
use std::path::{Path, PathBuf};
use std::process::exit;

use crate::color::ColorInfo;

pub fn save_image_format(image: &DynamicImage, out: &str, format: Option<&str>, overwrite: bool) {
    let mut out_path = PathBuf::from(out);
    let mut img_format = ImageFormat::Png;
    if format.is_some() {
        match format {
            Some(s) => {
                img_format = ImageFormat::from_extension(s)
                    .expect("Error obtaining image format from extension: ");
                let formats = img_format.extensions_str();

                out_path.set_extension(formats[0]);
            }
            _ => {
                let _ = img_format;
                eprintln!("Please choose a valid image format.");
                exit(0);
            }
        };
    } else {
        img_format = ImageFormat::from_path(&out_path)
            .expect("Error detecting image format from output path: ");
    }
    if !overwrite {
        check_overwrite(&out_path);
    }
    image
        .save_with_format(&out_path, img_format)
        .expect("File save error:");
}

#[derive(Debug, Default)]
pub struct Dimensions {
    pub x: u32,
    pub y: u32,
}

pub fn resize_image(
    image: &mut DynamicImage,
    dimensions: Dimensions,
    filter: String,
    preserve_aspect: bool,
) {
    let mut filter_type = FilterType::Nearest;
    let local_filter: &str = &filter;
    match local_filter.to_uppercase().as_str() {
        "NEAREST" => {
            filter_type = FilterType::Nearest;
        }
        "TRIANGLE" => {
            filter_type = FilterType::Triangle;
        }
        "GAUSSIAN" => {
            filter_type = FilterType::Gaussian;
        }
        "CATMULLROM" => {
            filter_type = FilterType::CatmullRom;
        }
        "LANCZOS" => {
            filter_type = FilterType::Lanczos3;
        }
        _ => {
            let _ = filter_type;
            eprintln!("Please pick a valid filter out of: Nearest, Triangle, Gaussian, CatmullRom and Lanczos");
            exit(0);
        }
    }

    *image = if preserve_aspect {
        image.resize(dimensions.x, dimensions.y, filter_type)
    } else {
        image.resize_exact(dimensions.x, dimensions.y, filter_type)
    }
}

pub fn remove_background(image: &mut DynamicImage) {
    let color_info = ColorInfo::from_image(image);

    if color_info.bit_depth == 8 {
        let mut img = image.to_rgba8();
        for p in img.pixels_mut() {
            if p[0] == 255 && p[1] == 255 && p[2] == 255 {
                p[3] = 0;
            }
        }
        *image = DynamicImage::ImageRgba8(img);
    } else if color_info.bit_depth == 16 {
        let mut img = image.to_rgba16();
        for p in img.pixels_mut() {
            if p[0] == 255 && p[1] == 255 && p[2] == 255 {
                p[3] = 0;
            }
        }
        *image = DynamicImage::ImageRgba16(img);
    } else {
        let mut img = image.to_rgba32f();
        for p in img.pixels_mut() {
            if p[0] == 255.0 && p[1] == 255.0 && p[2] == 255.0 {
                p[3] = 0.0;
            }
        }
        *image = DynamicImage::ImageRgba32F(img);
    }
}

fn check_overwrite(path: &Path) {
    if path.try_exists().expect("Error parsing output path") {
        let message = format!("Overwrite existing file: {:?}?", path.as_os_str().to_str());
        let confirm = Confirm::new().with_prompt(message).interact().unwrap();
        if !confirm {
            println!("Not overwriting existing file.");
            exit(0);
        }
    }
}
