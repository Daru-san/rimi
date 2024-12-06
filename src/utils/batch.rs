use image::ImageReader;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::exit;

pub fn check_batch(images: Vec<&str>) {
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
        eprintln!("Errors occured while parsing images:");
        for error in errors {
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
