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

pub fn create_paths(files: Vec<&str>, directory: &str, name_expr: Option<&str>) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;

    for file in files {
        i += 1;
        let mut file_path = PathBuf::from(file);

        if name_expr.is_some() {
            let expr = name_expr.map(|s| s.to_string()).unwrap();
            let ext_path = PathBuf::from(&expr);
            let extension = ext_path.extension().unwrap();
            let fname = format!("{}_{}", i, expr);
            file_path.set_file_name(fname);
            file_path.set_extension(extension);
        }

        let full_path = PathBuf::from(directory).join(file_path);
        paths.push(full_path.to_string_lossy().to_string());
    }

    paths
}
