use super::color::ColorInfo;
use image::imageops::FilterType;
use image::{self, DynamicImage, ImageFormat, ImageReader};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn open_image(image_path: &Path) -> Result<DynamicImage, String> {
    match ImageReader::open(image_path) {
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
) -> Result<(), String> {
    let mut out_path = PathBuf::from(out);
    let image_format;

    if let Some(format_extension) = format {
        image_format = match ImageFormat::from_extension(format_extension) {
            Some(format) => format,
            _ => {
                return Err(format!(
                    "Couldn't get image format from extension: {}",
                    format_extension
                ))
            }
        };
        let extension = image_format.extensions_str();
        if extension.is_empty() {
            return Err("Image format has no valid file extension".to_string());
        }
        out_path.set_extension(extension[0]);
    } else {
        image_format = match ImageFormat::from_path(&out_path) {
            Ok(format) => format,
            Err(_) => return Err("Could not obtain image format from output path".to_string()),
        }
    }

    match image.save_with_format(&out_path, image_format) {
        Ok(()) => Ok(()),
        Err(save_error) => Err(format!(
            "Error saving image file {:?}: {}",
            out_path, save_error
        )),
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
        let mut image8bit = image.to_rgba8();
        for p in image8bit.pixels_mut() {
            if p[0] == 255 && p[1] == 255 && p[2] == 255 {
                p[3] = 0;
            }
        }
        *image = DynamicImage::ImageRgba8(image8bit);
    } else if color_info.bit_depth == 16 {
        let mut image16bit = image.to_rgba16();
        for p in image16bit.pixels_mut() {
            if p[0] == 255 && p[1] == 255 && p[2] == 255 {
                p[3] = 0;
            }
        }
        *image = DynamicImage::ImageRgba16(image16bit);
    } else {
        let mut image32bit = image.to_rgba32f();
        for p in image32bit.pixels_mut() {
            if p[0] == 255.0 && p[1] == 255.0 && p[2] == 255.0 {
                p[3] = 0.0;
            }
        }
        *image = DynamicImage::ImageRgba32F(image32bit);
    }
}
