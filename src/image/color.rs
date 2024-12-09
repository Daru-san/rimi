use std::fmt::Display;
use std::str::FromStr;

use clap::ValueEnum;
use image::{ColorType, DynamicImage};

#[derive(Debug, Clone)]
pub struct ColorInfo {
    pub bit_depth: BitDepth,
    pub color_type: ColorTypeExt,
}

#[derive(Debug, Clone, Copy)]
pub enum BitDepth {
    B8 = 8,
    B16 = 16,
    B32 = 32,
}

impl Display for BitDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitDepth::B8 => write!(f, "{}", 8),
            BitDepth::B16 => write!(f, "{}", 16),
            BitDepth::B32 => write!(f, "{}", 32),
        }
    }
}

impl PartialEq for BitDepth {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl FromStr for BitDepth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num: u32 = s.parse::<u32>().unwrap();

        match num {
            8 => Ok(BitDepth::B8),
            16 => Ok(BitDepth::B16),
            32 => Ok(BitDepth::B32),
            _ => Err("Bit depth must be 8, 16 or 32.".into()),
        }
    }
}

#[derive(ValueEnum, Debug, Clone)]
pub enum ColorTypeExt {
    Rgb,
    RgbA,
    Luma,
    LumaA,
    Unknown,
}

impl Display for ColorTypeExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ColorInfo {
    pub fn new(color_type: ColorTypeExt, bit_depth: BitDepth) -> Self {
        ColorInfo {
            color_type,
            bit_depth,
        }
    }
    pub fn from_image(image: &DynamicImage) -> Self {
        match image.color() {
            ColorType::L8 => ColorInfo {
                bit_depth: BitDepth::B8,
                color_type: ColorTypeExt::Luma,
            },
            ColorType::La8 => ColorInfo {
                bit_depth: BitDepth::B8,
                color_type: ColorTypeExt::LumaA,
            },
            ColorType::L16 => ColorInfo {
                bit_depth: BitDepth::B16,
                color_type: ColorTypeExt::Luma,
            },
            ColorType::La16 => ColorInfo {
                bit_depth: BitDepth::B16,
                color_type: ColorTypeExt::LumaA,
            },
            ColorType::Rgb8 => ColorInfo {
                bit_depth: BitDepth::B8,
                color_type: ColorTypeExt::Rgb,
            },
            ColorType::Rgba8 => ColorInfo {
                bit_depth: BitDepth::B8,
                color_type: ColorTypeExt::RgbA,
            },
            ColorType::Rgb16 => ColorInfo {
                bit_depth: BitDepth::B16,
                color_type: ColorTypeExt::Rgb,
            },
            ColorType::Rgba16 => ColorInfo {
                bit_depth: BitDepth::B16,
                color_type: ColorTypeExt::Rgb,
            },
            ColorType::Rgb32F => ColorInfo {
                bit_depth: BitDepth::B32,
                color_type: ColorTypeExt::Rgb,
            },
            ColorType::Rgba32F => ColorInfo {
                bit_depth: BitDepth::B32,
                color_type: ColorTypeExt::RgbA,
            },
            _ => ColorInfo {
                bit_depth: BitDepth::B8,
                color_type: ColorTypeExt::Unknown,
            },
        }
    }
    pub fn convert_image(&self, image: &mut DynamicImage) {
        let color_type = self.to_color_type();
        let colored_image = match color_type {
            ColorType::L8 => DynamicImage::ImageLuma8(image.to_luma8()),
            ColorType::La8 => DynamicImage::ImageLumaA8(image.to_luma_alpha8()),
            ColorType::L16 => DynamicImage::ImageLuma16(image.to_luma16()),
            ColorType::La16 => DynamicImage::ImageLumaA16(image.to_luma_alpha16()),
            ColorType::Rgb8 => DynamicImage::ImageRgb8(image.to_rgb8()),
            ColorType::Rgba8 => DynamicImage::ImageRgba8(image.to_rgba8()),
            ColorType::Rgb16 => DynamicImage::ImageRgb16(image.to_rgb16()),
            ColorType::Rgba16 => DynamicImage::ImageRgba16(image.to_rgba16()),
            ColorType::Rgb32F => DynamicImage::ImageRgb32F(image.to_rgb32f()),
            ColorType::Rgba32F => DynamicImage::ImageRgba32F(image.to_rgba32f()),
            _ => image.clone(),
        };
        *image = colored_image;
    }
    fn to_color_type(&self) -> ColorType {
        match self.color_type {
            ColorTypeExt::Rgb => match self.bit_depth {
                BitDepth::B8 => ColorType::Rgb8,
                BitDepth::B16 => ColorType::Rgb16,
                BitDepth::B32 => ColorType::Rgb32F,
            },
            ColorTypeExt::RgbA => match self.bit_depth {
                BitDepth::B8 => ColorType::Rgba8,
                BitDepth::B16 => ColorType::Rgba16,
                BitDepth::B32 => ColorType::Rgba32F,
            },
            ColorTypeExt::Luma => match self.bit_depth {
                BitDepth::B8 => ColorType::L8,
                BitDepth::B16 => ColorType::L16,
                BitDepth::B32 => ColorType::L16,
            },
            ColorTypeExt::LumaA => match self.bit_depth {
                BitDepth::B8 => ColorType::La8,
                BitDepth::B16 => ColorType::La16,
                BitDepth::B32 => ColorType::La16,
            },
            ColorTypeExt::Unknown => ColorType::Rgb8,
        }
    }
}
