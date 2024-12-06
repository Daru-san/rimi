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
