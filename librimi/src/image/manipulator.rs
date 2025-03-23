use image::{DynamicImage, ImageReader};
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

pub fn open_image(image_path: &Path) -> Result<DynamicImage, String> {
    let mut file = match File::open(image_path) {
        Ok(file) => file,
        Err(file_error) => return Err(file_error.to_string()),
    };

    let len = match file.metadata() {
        Ok(data) => data.len(),
        Err(metadata_error) => return Err(metadata_error.to_string()),
    };

    let mut buffer = Vec::with_capacity(len as usize + 1);

    match file.read_to_end(&mut buffer) {
        Ok(_) => (),
        Err(read_error) => return Err(read_error.to_string()),
    }

    let reader = Cursor::new(buffer);

    match ImageReader::new(reader).with_guessed_format() {
        Ok(reader) => match reader.decode() {
            Ok(image) => Ok(image),
            Err(decode_error) => Err(format!(
                "Error decoding image {:?}: {}",
                image_path, decode_error
            )),
        },
        Err(decode_error) => Err(format!(
            "Error decoding image {:?}: {}",
            image_path, decode_error
        )),
    }
}
