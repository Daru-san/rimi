use std::path::{Path, PathBuf};

pub fn create_paths(
    files: Vec<PathBuf>,
    destination: PathBuf,
    name_expr: Option<&str>,
    image_format: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    let mut paths: Vec<PathBuf> = Vec::new();

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
        Some(format) => Some(ImageFormat::from_extension(format).unwrap()),
        None => name_expr.map(|expression| ImageFormat::from_path(expression).unwrap()),
    };

    for (index, file) in files.iter().enumerate() {
        let file_name = match name_expr {
            Some(expression) => {
                format!("{expression}_{index}")
            }
            None => {
                if let Some(filename) = file.file_name() {
                    filename.to_string_lossy().to_string()
                } else {
                    format!("image_{index}")
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

        if is_formatted {
            paths.push(path);
        } else {
            return Err("Error formatting output file".into());
        }
    }
    Ok(paths)
}

