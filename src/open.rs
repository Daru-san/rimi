use image::{error::LimitErrorKind, DynamicImage, ImageError, ImageReader};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::exit;

pub fn open_image(image_path: PathBuf) -> DynamicImage {
    let path = image_path.clone();
    let path_str = path.as_os_str();
    let result = ImageReader::open(image_path);

    match result {
        Ok(decoded_result) => match decoded_result.decode() {
            Ok(image) => image,
            Err(e) => {
                match e {
                    ImageError::Decoding(_) => {
                        eprintln!("{:?}: Error decoding image: {}", path_str, e);
                    }
                    ImageError::IoError(e) => {
                        eprintln!("{:?}: Io error: {}", path_str, e);
                    }
                    ImageError::Limits(e) => match e.kind() {
                        LimitErrorKind::DimensionError => {
                            eprintln!("{:?}: Invalid dimensions: {}", path_str, e);
                        }
                        LimitErrorKind::InsufficientMemory => {
                            eprintln!("{:?}: Insufficient memory: {}", path_str, e);
                        }
                        _ => {
                            eprintln!("{:?}: Error decoding image: {}", path_str, e);
                        }
                    },
                    _ => {
                        eprintln!("{:?}: Error decoding image: {}", path_str, e);
                    }
                };
                exit(0)
            }
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::PermissionDenied => {
                    eprintln!(
                        "{:?}: Operation not permitted. Please check file permissions.",
                        path_str
                    );
                }
                ErrorKind::NotFound => {
                    eprintln!("{:?}: File could not be found.", path_str);
                }
                _ => {
                    eprintln!("{:?}: Error opening image: {}", path_str, e);
                }
            };
            exit(0);
        }
    }
}
