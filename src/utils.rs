use dialoguer::Confirm;
use image::imageops::FilterType;
use image::{self, DynamicImage, ImageFormat};
use std::path::PathBuf;
use std::process::exit;


pub fn save_image_format(image: &DynamicImage, out: &str, format: Option<String>, overwrite: bool) {
    let mut out_path = PathBuf::from(out);
    let mut img_format = ImageFormat::Png;
    if format.is_some() {
        let local_format: &str = &format
            .map(|s| s.to_string())
            .expect("Something strange happened");

        match local_format.to_uppercase().as_str() {
            "PNG" => {
                img_format = ImageFormat::Png;
                out_path.set_extension("png");
            }
            "JPG" => {
                img_format = ImageFormat::Jpeg;
                out_path.set_extension("jpg");
            }
            "JPEG" => {
                img_format = ImageFormat::Jpeg;
                out_path.set_extension("jpg");
            }
            "WEBP" => {
                img_format = ImageFormat::WebP;
                out_path.set_extension("webp");
            }
            "ICO" => {
                img_format = ImageFormat::Ico;
                out_path.set_extension("ico");
            }
            "GIF" => {
                img_format = ImageFormat::Gif;
                out_path.set_extension("gif");
            }
            "AVIF" => {
                img_format = ImageFormat::Avif;
                out_path.set_extension("avif");
            }
            _ => {
                let _ = img_format;
                eprintln!("Please choose one of png, jpg, jpeg, webp, ico, gif or avif.");
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
        "LANCZOS" => {
            filter_type = FilterType::Lanczos3;
        }
        _ => {
            let _ = filter_type;
            eprintln!("Please pick a valid filter out of: Nearest, Triangle, Gaussian, CatmullRom and Lanczos");
            exit(0);
        }
    }
    image.resize(dimensions.x, dimensions.y, filter_type);
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
