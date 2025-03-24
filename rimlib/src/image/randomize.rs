use image::{ColorType, DynamicImage, PixelWithColorType};

use super::color::ColorData;

/// Randomizer
///
/// This trait can only be implemented on data implementing the `Sized` and `Clone` traits.
/// Implemented for `DynamicImage` by default.
///
/// Randomizes the data in images.
/// This trait is mainly meant for testing purposes but can be used in other ways.
/// These functions all return a new instance of the image implementing the trait.
pub trait Randomizer: Sized + Clone {
    /// Fills image with completely random data
    /// Acts as a combination of all of the other functions
    ///
    /// Returns a new instance of the image
    fn randomize_all(&self) -> Result<Self, RandomizerError>;

    /// Changes the hue of the image based on random generated values
    /// Returns a new instance of the image
    fn randomize_hue(&self) -> Result<Self, RandomizerError>;

    /// Changes the saturation of the image based on random generated values
    /// Returns a new instance of the image
    fn randomize_saturation(&self) -> Result<Self, RandomizerError>;

    /// Changes the size to one random size of range between provided range parameters
    /// Returns a new instance of the image
    fn randomize_size(
        &self,
        min_width: u32,
        min_height: u32,
        max_width: u32,
        max_height: u32,
        filter_type: Option<FilterType>,
    ) -> Result<Self, RandomizerError>;

    /// Changes the coloration of the image based on a color type provided as a parameter
    /// Color types and bit-depths are not changed within this function.
    /// Returns a new instance of the image
    fn randomize_color<X>(&self, color_type: X) -> Result<Self, RandomizerError>
    where
        X: ColorData;
}

/// These implementations are created to make it easy to work with iamage types such as
/// ImageBuffers by using DynamicImage::from(buffer), which removes the need to implement
/// this for every image type manually
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
