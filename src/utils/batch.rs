use image::ImageReader;
use std::error::Error;
use std::fmt;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct BatchError(pub Vec<String>);

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Errors occured while parsing images: {}", self.0.len())?;
        for err in &self.0 {
            write!(f, "{}", err)?
        }
        Ok(())
    }
}

impl Error for BatchError {}

pub fn check_batch(images: Vec<&str>) -> Result<(), Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    for image in images {
        let path = PathBuf::from(image);
        match ImageReader::open(image) {
            Ok(reader) => match reader.decode() {
                Ok(_) => (),
                Err(e) => errors.push(format!("{:?}: {}", path, e)),
            },
            Err(e) => match e.kind() {
                ErrorKind::PermissionDenied => {
                    errors.push(format!("{:?}: File access not permitted", path))
                }
                ErrorKind::NotFound => errors.push(format!("{:?}: File not found", path)),
                _ => errors.push(format!("{:?}: IO error: {}", path, e)),
            },
        }
    }
    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(())
    }
}

pub fn create_paths(
    files: Vec<&str>,
    directory: PathBuf,
    name_expr: Option<&str>,
) -> Result<Vec<String>, String> {
    let mut paths: Vec<String> = Vec::new();
    let dest_dir = directory;

    if !dest_dir.is_dir() {
        return Err(format!(
            "Directory {:?} does not exist.",
            dest_dir.as_os_str()
        ));
    }

    for (index, file) in files.iter().enumerate() {
        let mut path = dest_dir.join(file);

        let file_name = match name_expr {
            Some(expr) => {
                let expr_path = Path::new(expr);
                if let Some(extension) = expr_path.extension() {
                    let new_name =
                        format!("{}_{}.{}", expr, index + 1, extension.to_str().unwrap());
                    new_name
                } else {
                    return Err(format!(
                        "File expression {} does not have an extension.",
                        expr
                    ));
                }
            }
            None => path.file_name().unwrap().to_str().unwrap().to_string(),
        };
        path.set_file_name(file_name);

        paths.push(path.to_string_lossy().to_string());
    }

    Ok(paths)
}
