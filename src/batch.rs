use image::{error::LimitErrorKind, ImageError, ImageReader};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::exit;

pub fn check_batch(images: Vec<&str>) {
    let mut bad_images: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for image in images {
        let path = PathBuf::from(image);

        let result = ImageReader::open(image);
        match result {
            Ok(decoded_result) => match decoded_result.decode() {
                Ok(_) => {}
                Err(e) => match e {
                    ImageError::Decoding(_) => {
                        errors.push(format!(
                            "{:?}: Error decoding image.",
                            path.as_os_str().to_str()
                        ));
                        bad_images.push(image.to_string());
                    }
                    ImageError::IoError(e) => {
                        errors.push(format!(
                            "{:?}: An IO error occured opening the image: {:?}",
                            path.as_os_str().to_str(),
                            e.to_string()
                        ));
                        bad_images.push(image.to_string());
                    }
                    ImageError::Limits(e) => match e.kind() {
                        LimitErrorKind::DimensionError => {
                            errors.push(format!(
                                "{:?}: Image dimensions are too large: {}",
                                path.as_os_str(),
                                e
                            ));
                            bad_images.push(image.to_string());
                        }
                        LimitErrorKind::InsufficientMemory => {
                            errors.push(format!(
                                "{:?}: Insufficient memory to open image.",
                                path.as_os_str()
                            ));
                            bad_images.push(image.to_string());
                        }
                        _ => {
                            errors
                                .push(format!("{:?}: An error occured: {}", path.as_os_str(), e,));
                            bad_images.push(image.to_string());
                        }
                    },
                    _ => {
                        errors.push(format!("{:?}: An error occured: {}", path.as_os_str(), e));
                        bad_images.push(image.to_string());
                    }
                },
            },
            Err(e) => match e.kind() {
                ErrorKind::PermissionDenied => {
                    errors.push(format!(
                        "{:?}: File access not permitted on file. Please check file permissions.",
                        path.as_os_str().to_str()
                    ));
                    bad_images.push(image.to_string());
                }
                ErrorKind::NotFound => {
                    errors.push(format!(
                        "{:?}: File not found in current directory. Please ensure it exists.",
                        path.as_os_str()
                    ));
                }
                _ => {
                    errors.push(format!(
                        "{:?}: An io error occured: {}",
                        path.as_os_str(),
                        e,
                    ));
                }
            },
        }
    }
    if !errors.is_empty() {
        eprintln!("Errors occured while parsing images: ");
        for error in errors.clone().into_iter() {
            eprintln!("{}", error);
        }
        exit(0);
    }
}
