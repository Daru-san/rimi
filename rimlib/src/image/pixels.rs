use image::{ColorType, DynamicImage, GenericImage, ImageBuffer, Luma, LumaA, Rgb, Rgba};

use super::color::ColorData;

pub enum ImageBufferData {
    Rgb8(ImageBuffer<Rgb<u8>, Vec<u8>>),
    Rgba8(ImageBuffer<Rgba<u8>, Vec<u8>>),
    Rgb16(ImageBuffer<Rgba<u16>, Vec<u16>>),
    Rgba16(ImageBuffer<Rgba<u16>, Vec<u16>>),
    Rgb32f(ImageBuffer<Rgb<f32>, Vec<f32>>),
    Rgba32f(ImageBuffer<Rgba<f32>, Vec<f32>>),
    Luma8(ImageBuffer<Luma<u8>, Vec<u8>>),
    LumaA8(ImageBuffer<LumaA<u8>, Vec<u8>>),
    Luma16(ImageBuffer<Luma<u16>, Vec<u16>>),
    LumaA16(ImageBuffer<LumaA<u16>, Vec<u16>>),
}

/// Encompasses a trait used for converting the color type of all images
/// into a chosen type implementing the `ColorData` trait
pub trait PixelConvert {
    /// Converts the image or buffer into one of the chosen type.
    /// Returns an instance of `ImageBufferData` with the buffer as an inner.
    /// Buffers can be obtained through matching or if-statements.
    fn convert_color_to<C>(&self, color_info: C) -> ImageBufferData
    where
        C: ColorData;
}

impl PixelConvert for DynamicImage {
    fn convert_color_to<C>(&self, color_info: C) -> ImageBufferData
    where
        C: ColorData,
    {
        let cltype = color_info.color_type();
        match cltype {
            ColorType::L8 => ImageBufferData::Luma8(self.to_luma8()),
            ColorType::La8 => ImageBufferData::LumaA8(self.to_luma_alpha8()),
            ColorType::L16 => ImageBufferData::Luma16(self.to_luma16()),
            ColorType::La16 => ImageBufferData::LumaA16(self.to_luma_alpha16()),
            ColorType::Rgb8 => ImageBufferData::Rgb8(self.to_rgb8()),
            ColorType::Rgba8 => ImageBufferData::Rgba8(self.to_rgba8()),
            ColorType::Rgb16 => ImageBufferData::Rgb16(self.to_rgba16()),
            ColorType::Rgba16 => ImageBufferData::Rgba16(self.to_rgba16()),
            ColorType::Rgb32F => ImageBufferData::Rgb32f(self.to_rgb32f()),
            ColorType::Rgba32F => ImageBufferData::Rgba32f(self.to_rgba32f()),
            _ => unimplemented!(),
        }
    }
}
