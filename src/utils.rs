use image::imageops::FilterType;
use image::{self, DynamicImage, ImageFormat};
use std::path::PathBuf;
use std::process::exit;

pub fn save_image(image: &DynamicImage, path: &str) {
    image.save(path).expect("File save error:");
}

pub fn save_image_format(image: DynamicImage, out: &str, format: Option<String>) {
    let mut path = PathBuf::from(out);
    if format.is_some() {
        let mut img_format = ImageFormat::Png;
        let local_format: &str = &format
            .map(|s| s.to_string())
            .expect("Something strange happened");

        match local_format.to_uppercase().as_str() {
            "PNG" => {
                img_format = ImageFormat::Png;
                path.set_extension("png");
            }
            "JPG" => {
                img_format = ImageFormat::Jpeg;
                path.set_extension("jpg");
            }
            "JPEG" => {
                img_format = ImageFormat::Jpeg;
                path.set_extension("jpg");
            }
            "WEBP" => {
                img_format = ImageFormat::WebP;
                path.set_extension("webp");
            }
            "ICO" => {
                img_format = ImageFormat::Ico;
                path.set_extension("ico");
            }
            "GIF" => {
                img_format = ImageFormat::Gif;
                path.set_extension("gif");
            }
            "AVIF" => {
                img_format = ImageFormat::Avif;
                path.set_extension("avif");
            }
            _ => {
                let _ = img_format;
                eprintln!("Please choose one of png, jpg, jpeg, webp, ico, gif or avif.");
                exit(0);
            }
        };
        image
            .save_with_format(path, img_format)
            .expect("File save error:");
    } else {
        image.save(path).expect("File save error:");
    }
}

#[derive(Debug, Default)]
pub struct Dimensions {
    pub x: u32,
    pub y: u32,
}

pub fn resize_image(image: &mut DynamicImage, dimensions: Dimensions, filter: String) {
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
        "LANCZOS3" => {
            filter_type = FilterType::Lanczos3;
        }
        _ => {
            let _ = filter_type;
            eprintln!("Please pick a valid filter out of: Nearest, Triangle, Gaussian, CatmullRom and Lanczos3");
            exit(0);
        }
    }
    image.resize(dimensions.x, dimensions.y, filter_type);
}
