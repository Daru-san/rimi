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

pub fn create_paths(
    files: Vec<PathBuf>,
    destination: PathBuf,
    name_expr: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let dest_dir = destination;

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

        paths.push(path);
    }

    Ok(paths)
}
