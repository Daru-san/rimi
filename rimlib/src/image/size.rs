use image::{DynamicImage, imageops::FilterType};

#[derive(Debug, Default)]
pub struct Dimensions {
    pub x: u32,
    pub y: u32,
}

pub fn resize_image(
    image: DynamicImage,
    dimensions: Dimensions,
    filter: String,
    preserve_aspect: bool,
) -> Result<DynamicImage, String> {
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

    if preserve_aspect {
        Ok(image.resize(dimensions.x, dimensions.y, filter_type))
    } else {
        Ok(image.resize_exact(dimensions.x, dimensions.y, filter_type))
    }
}
