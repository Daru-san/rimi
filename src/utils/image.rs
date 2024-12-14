use super::color::ColorInfo;
use dialoguer::Confirm;
use image::imageops::FilterType;
use image::{self, DynamicImage, ImageFormat, ImageReader};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn open_image(image_path: &Path) -> Result<DynamicImage, String> {
    match ImageReader::open(&image_path) {
        Ok(reader) => match reader.decode() {
            Ok(image) => Ok(image),
            Err(decode_error) => Err(format!(
                "Error decoding image {:?}: {}",
                image_path, decode_error
            )),
        },
        Err(file_error) => match file_error.kind() {
            ErrorKind::NotFound => Err(format!("File not found {:?}", image_path)),
            ErrorKind::PermissionDenied => Err(format!("Permission denied: {:?}", image_path)),
            _ => Err(format!(
                "Error opening image {:?}: {}",
                image_path, file_error
            )),
        },
    }
}

pub fn save_image_format(
    image: &DynamicImage,
    out: &Path,
    format: Option<&str>,
    overwrite: bool,
) -> Result<(), String> {
    let mut out_path = PathBuf::from(out);
    let img_format;

    if let Some(path) = path {
        img_format = match ImageFormat::from_extension(path) {
            Some(format) => format,
            _ => return Err(path!("Couldn't get image format from extension: {}", path)),
        };
        let extension = img_format.extensions_str();
        if extension.is_empty() {
            return Err("Image format has no valid file extension".to_string());
        }
        out_path.set_extension(extension[0]);
    } else {
        img_format = match ImageFormat::from_path(&out_path) {
            Ok(format) => format,
            Err(_) => return Err("Could not obtain image format from output path".to_string()),
        }
    }

    match overwrite_if_existing(&out_path, overwrite) {
        Ok(overwrite) => {
            if overwrite {
                match image.save_with_format(&out_path, img_format) {
                    Ok(()) => Ok(()),
                    Err(save_error) => Err(format!(
                        "Error saving image file {:?}: {}",
                        out_path, save_error
                    )),
                }
            } else {
                Err("Not overriding existing file.".to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
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
) -> Result<(), String> {
    let filter_type;
    match filter.to_uppercase().as_str() {
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
            return Err("Please pick a valid filter out of: Nearest, Triangle, Gaussian, CatmullRom and Lanczos".to_string())
        }
    }

    *image = if preserve_aspect {
        image.resize(dimensions.x, dimensions.y, filter_type)
    } else {
        image.resize_exact(dimensions.x, dimensions.y, filter_type)
    };

    Ok(())
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

fn overwrite_if_existing(path: &Path, do_overwrite: bool) -> Result<bool, String> {
    if do_overwrite {
        return Ok(true);
    }
    match path.try_exists() {
        Ok(path_exists) => match path_exists {
            true => {
                let message = format!("Overwrite existing file: {:?}?", path.to_string_lossy());
                let confirm = Confirm::new().with_prompt(message).interact();
                match confirm {
                    Ok(val) => Ok(val),
                    Err(confirm_error) => Err(confirm_error.to_string()),
                }
            }
            false => Ok(true),
        },
        Err(path_error) => Err(path_error.to_string()),
    }
}
