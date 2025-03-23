use std::path::{Path, PathBuf};

use dialoguer::Confirm;
use image::ImageFormat;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

pub fn create_paths(
    paths: Vec<PathBuf>,
    destination: PathBuf,
    name_expr: Option<&str>,
    image_format: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    let mut paths = paths;
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

    let set_expr = |acc: usize, name: String| -> String {
        let mut name = name;
        name.push_str(&format!("_{acc}"));
        name
    };
    let set_extension = |file: &PathBuf, path: &mut PathBuf| -> Result<bool, String> {
        match image_format {
            Some(image_format) => Ok(path.set_extension(image_format.extensions_str()[0])),
            None => match file.extension() {
                Some(extension) => Ok(path.set_extension(extension)),
                None => Err("File does not have an extension?".into()),
            },
        }
    };

    paths.par_iter_mut().enumerate().for_each(|(acc, path)| {
        let file_name = match name_expr {
            Some(expression) => set_expr(acc, expression.to_string()),
            None => {
                if let Some(filename) = path.file_name() {
                    filename.to_string_lossy().to_string()
                } else {
                    String::from("image")
                }
            }
        };
        let dest = destination.to_path_buf();
        let mut dest = dest.join(file_name);
        let _ = set_extension(path, &mut dest);
        *path = (*dest).to_path_buf();
    });
    Ok(paths)
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
