use image::{ColorType, DynamicImage, PixelWithColorType};

use super::color::ColorData;

pub trait Randomizer {
    /// Randomizer
    ///
    /// Fills image with random data
    fn randomize_all(&self) -> Result<String, RandomizerError>;

    fn randomize_hue(&self) -> Result<String, RandomizerError>;

    fn randomize_saturation(&self) -> Result<String, RandomizerError>;

    fn randomize_size(
        &self,
        min_width: u32,
        min_height: u32,
        max_width: u32,
        max_height: u32,
    ) -> Result<String, RandomizerError>;

    fn randomize_color<X>(&self, color_type: X) -> Result<String, RandomizerError>
    where
        X: ColorData;
}

impl Randomizer for DynamicImage {
    fn randomize_all(&self) -> Result<String, RandomizerError> {}
    fn randomize_hue(&self) -> Result<String, RandomizerError> {}
    fn randomize_saturation(&self) -> Result<String, RandomizerError> {}
    fn randomize_size(
        &self,
        min_width: u32,
        min_height: u32,
        max_width: u32,
        max_height: u32,
    ) -> Result<String, RandomizerError> {
    }

    fn randomize_color<X>(&self, color_data: X) -> Result<String, RandomizerError>
    where
        X: ColorData,
    {
        let data = color_data.color_info();
    }
}

pub enum RandomizerError {}
