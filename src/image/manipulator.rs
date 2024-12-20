use super::color::ColorInfo;
use image::imageops::FilterType;
use image::{load_from_memory, DynamicImage, ImageFormat, ImageReader};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::fs::File;
use std::io::{BufWriter, Cursor, Read};
use std::mem::take;
use std::path::{Path, PathBuf};

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

pub fn convert_image(
    image: &mut DynamicImage,
    format: Option<&str>,
) -> Result<DynamicImage, String> {
    let image_format = image_format(format, None)?;

    // Avif cannot be decoded in memory,
    // hence we return and leave it to save_image_format()
    // TODO: Fix Avif decoding
    if image_format == ImageFormat::Avif {
        return Ok(take(image));
    }

    let mut writer = Cursor::new(Vec::with_capacity(image.as_bytes().len() + 1));

    match image.write_to(&mut writer, image_format) {
        Ok(()) => (),
        Err(e) => return Err(e.to_string()),
    };

    let result = load_from_memory(&writer.into_inner());

    match result {
        Ok(mut image) => Ok(take(&mut image)),
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

#[derive(Debug, Default)]
pub struct Dimensions {
    pub x: u32,
    pub y: u32,
}

pub fn resize_image(
    image: &mut DynamicImage,
    dimensions: Dimensions,
    filter: String,
    preserve_aspect: bool,
) -> Result<(), String> {
    let filter_type;
    match filter.to_uppercase().as_str() {
        "NEAREST" => {
            filter_type = FilterType::Nearest;
        }
        "TRIANGLE" => {
            filter_type = FilterType::Triangle;
        }
        "GAUSSIAN" => {
            filter_type = FilterType::Gaussian;
        }
        "CATMULLROM" => {
            filter_type = FilterType::CatmullRom;
        }
        "LANCZOS" => {
            filter_type = FilterType::Lanczos3;
        }
        _ => {
            return Err("Please pick a valid filter out of: Nearest, Triangle, Gaussian, CatmullRom and Lanczos".to_string())
        }
    }

    *image = if preserve_aspect {
        image.resize(dimensions.x, dimensions.y, filter_type)
    } else {
        image.resize_exact(dimensions.x, dimensions.y, filter_type)
    };

    Ok(())
}

pub fn remove_background(image: &mut DynamicImage) {
    use super::color::BitDepth::{B16, B32, B8};
    let color_info = ColorInfo::from_image(image);

    match color_info.bit_depth {
        B8 => {
            let mut image8bit = image.to_rgba8();
            image8bit.pixels_mut().par_bridge().for_each(|pixel| {
                if pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 255 {
                    pixel[3] = 0;
                }
            });
            *image = DynamicImage::ImageRgba8(image8bit);
        }

        B16 => {
            let mut image16bit = image.to_rgba16();
            image16bit.pixels_mut().par_bridge().for_each(|pixel| {
                if pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 255 {
                    pixel[3] = 0;
                }
            });
            *image = DynamicImage::ImageRgba16(image16bit);
        }

        B32 => {
            let mut image32bit = image.to_rgba32f();
            image32bit.pixels_mut().par_bridge().for_each(|pixel| {
                if pixel[0] == 255.0 && pixel[1] == 255.0 && pixel[2] == 255.0 {
                    pixel[3] = 0.0;
                }
            });
            *image = DynamicImage::ImageRgba32F(image32bit);
        }
    }
}
