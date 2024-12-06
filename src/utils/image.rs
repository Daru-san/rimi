use dialoguer::Confirm;
use image::imageops::FilterType;
use image::{self, DynamicImage, ImageFormat};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::exit;

use crate::color::ColorInfo;

use image::{error::LimitErrorKind, ImageError, ImageReader};
use std::io::ErrorKind;

pub fn open_image(image_path: PathBuf) -> Result<DynamicImage, Box<dyn Error>> {
    let path = image_path.clone();
    let path_str = path.as_os_str();
    let result = ImageReader::open(image_path);

    match result {
        Ok(decoded_result) => match decoded_result.decode() {
            Ok(image) => Ok(image),
            Err(e) => {
                match e {
                    ImageError::Decoding(_) => {
                        return Err(format!("{:?}: Error decoding image: {}", path_str, e).into())
                    }
                    ImageError::IoError(e) => {
                        return Err(format!("{:?}: Io error: {}", path_str, e).into())
                    }
                    ImageError::Limits(e) => match e.kind() {
                        LimitErrorKind::DimensionError => {
                            return Err(format!("{:?}: Invalid dimensions: {}", path_str, e).into())
                        }
                        LimitErrorKind::InsufficientMemory => {
                            return Err(format!("{:?}: Insufficient memory: {}", path_str, e).into())
                        }
                        _ => {
                            return Err(
                                format!("{:?}: Error decoding image: {}", path_str, e).into()
                            )
                        }
                    },
                    _ => return Err(format!("{:?}: Error decoding image: {}", path_str, e).into()),
                };
            }
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::PermissionDenied => {
                    return Err(format!(
                        "{:?}: Operation not permitted. Please check file permissions.",
                        path_str
                    )
                    .into())
                }
                ErrorKind::NotFound => {
                    return Err(format!("{:?}: File could not be found.", path_str).into())
                }
                _ => return Err(format!("{:?}: Error opening image: {}", path_str, e).into()),
            };
        }
    }
}

pub fn save_image_format(
    image: &DynamicImage,
    out: &str,
    format: Option<&str>,
    overwrite: bool,
) -> Result<(), Box<dyn Error>> {
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
                return Err(format!("Please choose a valid image format.").into());
            }
        };
    } else {
        img_format = ImageFormat::from_path(&out_path)?;
    }
    if !overwrite {
        check_overwrite(&out_path);
    }
    image.save_with_format(&out_path, img_format)?;
    Ok(())
}
pub fn resize_image(
    image: &mut DynamicImage,
    dimensions: Dimensions,
    filter: String,
    preserve_aspect: bool,
) -> Result<(), Box<dyn Error>> {
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
            return Err("Please pick a valid filter out of: Nearest, Triangle, Gaussian, CatmullRom and Lanczos".into());
        }
    }

    *image = if preserve_aspect {
        image.resize(dimensions.x, dimensions.y, filter_type)
    } else {
        image.resize_exact(dimensions.x, dimensions.y, filter_type)
    };

    Ok(())
}
