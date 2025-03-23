use image::{DynamicImage, ImageFormat, load_from_memory};
use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};

fn image_format(format: Option<&str>, path: Option<&Path>) -> Result<ImageFormat, String> {
    if let Some(format_extension) = format {
        match ImageFormat::from_extension(format_extension) {
            Some(format) => Ok(format),
            _ => Err(format!(
                "Couldn't get image format from extension: {}",
                format_extension
            )),
        }
    } else {
        match ImageFormat::from_path(path.unwrap()) {
            Ok(format) => Ok(format),
            Err(_) => Err("Could not obtain image format from output path".to_string()),
        }
    }
}

pub fn convert_image(image: DynamicImage, format: Option<&str>) -> Result<DynamicImage, String> {
    let image_format = if let Some(format_extension) = format {
        match ImageFormat::from_extension(format_extension) {
            Some(format) => match format {
                // Avif cannot be decoded in memory yet,
                // hence we return and leave it to save_image_format()
                ImageFormat::Avif => return Ok(image),
                other_fmt => other_fmt,
            },
            _ => {
                return Err(format!(
                    "Could not get image format from extension: {format_extension}"
                ));
            }
        }
    } else {
        return Ok(image);
    };

    let mut writer = Cursor::new(Vec::with_capacity(image.as_bytes().len() + 1));

    match image.write_to(&mut writer, image_format) {
        Ok(()) => (),
        Err(e) => return Err(e.to_string()),
    };

    let result = load_from_memory(&writer.into_inner());

    match result {
        Ok(image) => Ok(image),
        Err(e) => Err(e.to_string()),
    }
}

pub fn save_image_format(
    image: &DynamicImage,
    out: &Path,
    format: Option<&str>,
) -> Result<(), String> {
    let mut out_path = PathBuf::from(out);
    let image_format = image_format(format, Some(out))?;

    let extension = image_format.extensions_str();

    if extension.is_empty() {
        return Err("Image format has no valid file extension".to_string());
    }

    out_path.set_extension(extension[0]);

    let output_file = match File::create(&out_path) {
        Ok(file) => file,
        Err(io_error) => return Err(format!("Error saving image {:?}: {}", out_path, io_error)),
    };

    let mut buffer = BufWriter::with_capacity(image.as_bytes().len() + 1, output_file);

    match image.write_to(&mut buffer, image_format) {
        Ok(()) => Ok(()),
        Err(save_error) => Err(format!(
            "Error saving image file {:?}: {}",
            out_path, save_error
        )),
    }
}
