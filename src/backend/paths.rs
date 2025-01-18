use std::path::{Path, PathBuf};

use dialoguer::Confirm;
use image::ImageFormat;

pub fn create_path(
    file: PathBuf,
    destination: PathBuf,
    name_expr: Option<&str>,
    image_format: Option<&str>,
) -> Result<PathBuf, String> {
    if destination.is_file() {
        return Err(format!(
            "Chosen path is a file, please choose a directory: {}",
            destination.as_os_str().to_string_lossy()
        ));
    }

    if !destination.is_dir() {
        return Err(format!(
            "Directory {} does not exist.",
            destination.as_os_str().to_string_lossy()
        ));
    }

    let image_format = match image_format {
        Some(format) => match ImageFormat::from_extension(format) {
            Some(image_format) => Some(image_format),
            None => return Err(format!("Image format provided is not valid: {}", format)),
        },
        None => match name_expr {
            Some(expression) => match ImageFormat::from_path(expression) {
                Ok(image_format) => Some(image_format),
                Err(error) => return Err(error.to_string()),
            },
            None => None,
        },
    };

    let file_name = match name_expr {
        Some(expression) => expression.to_string(),
        None => {
            if let Some(filename) = file.file_name() {
                filename.to_string_lossy().to_string()
            } else {
                String::from("image")
            }
        }
    };

    let path = destination.to_path_buf();
    let mut path = path.join(file_name);

    let is_formatted = match image_format {
        Some(image_format) => path.set_extension(image_format.extensions_str()[0]),
        None => match file.extension() {
            Some(extension) => path.set_extension(extension),
            None => return Err("File does not have an extension?".into()),
        },
    };

    let mut i = 0;
    while let Ok(exists) = path.try_exists() {
        if exists {
            if let Some(last) = path.iter().last() {
                if *last != *i.to_string() {
                    path.extend([i.to_string()].iter());
                } else {
                    i += 1;
                }
            }
        }
    }

    if is_formatted {
        Ok(path)
    } else {
        Err("Error formatting output file".into())
    }
}

pub fn prompt_overwrite_single(path: &Path) -> Result<(), String> {
    println!("Overwrite existing file: {}?", path.to_string_lossy());
    let confirm = Confirm::new()
        .with_prompt("Overwrite this file?")
        .interact();
    match confirm {
        Ok(overwrite) => match overwrite {
            true => Ok(()),
            false => Err("Not overwriting existing file".to_string()),
        },
        Err(confirm_error) => Err(confirm_error.to_string()),
    }
}
