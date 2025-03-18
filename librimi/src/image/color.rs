use std::fmt::Display;
use std::str::FromStr;

use image::{ColorType, DynamicImage};

pub trait ColorData {
    fn color_info(&self) -> ColorInfo;
    fn color_type(&self) -> ColorType;
}

impl ColorData for ColorInfo {
    fn color_info(&self) -> ColorInfo {
        *self
    }
    fn color_type(&self) -> ColorType {
        self.to_color_type()
    }
}
impl ColorData for ColorType {
    fn color_info(&self) -> ColorInfo {
        ColorInfo::from(*self)
    }
    fn color_type(&self) -> ColorType {
        *self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorInfo {
    pub bit_depth: BitDepth,
    pub color_space: ColorSpace,
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

#[derive(Debug, Clone, Copy)]
pub enum ColorSpace {
    Rgb,
    RgbA,
    Luma,
    LumaA,
    Unknown,
}

impl Display for ColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for ColorInfo {
    fn default() -> Self {
        Self {
            bit_depth: BitDepth::B8,
            color_space: ColorSpace::Rgb,
        }
    }
}

impl ColorInfo {
    pub fn new(color_space: &ColorSpace, bit_depth: &BitDepth) -> Self {
        ColorInfo {
            color_space: *color_space,
            bit_depth: *bit_depth,
        }
    }
    pub fn from_image(image: &DynamicImage) -> Self {
        match image.color() {
            ColorType::L8 => ColorInfo {
                bit_depth: BitDepth::B8,
                color_space: ColorSpace::Luma,
            },
            ColorType::La8 => ColorInfo {
                bit_depth: BitDepth::B8,
                color_space: ColorSpace::LumaA,
            },
            ColorType::L16 => ColorInfo {
                bit_depth: BitDepth::B16,
                color_space: ColorSpace::Luma,
            },
            ColorType::La16 => ColorInfo {
                bit_depth: BitDepth::B16,
                color_space: ColorSpace::LumaA,
            },
            ColorType::Rgb8 => ColorInfo {
                bit_depth: BitDepth::B8,
                color_space: ColorSpace::Rgb,
            },
            ColorType::Rgba8 => ColorInfo {
                bit_depth: BitDepth::B8,
                color_space: ColorSpace::RgbA,
            },
            ColorType::Rgb16 => ColorInfo {
                bit_depth: BitDepth::B16,
                color_space: ColorSpace::Rgb,
            },
            ColorType::Rgba16 => ColorInfo {
                bit_depth: BitDepth::B16,
                color_space: ColorSpace::Rgb,
            },
            ColorType::Rgb32F => ColorInfo {
                bit_depth: BitDepth::B32,
                color_space: ColorSpace::Rgb,
            },
            ColorType::Rgba32F => ColorInfo {
                bit_depth: BitDepth::B32,
                color_space: ColorSpace::RgbA,
            },
            _ => ColorInfo::default(),
        }
    }
    pub fn convert_image(&self, image: DynamicImage) -> DynamicImage {
        let color_type = self.to_color_type();
        match color_type {
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
            _ => image,
        }
    }
    fn to_color_type(&self) -> ColorType {
        match self.color_space {
            ColorSpace::Rgb => match self.bit_depth {
                BitDepth::B8 => ColorType::Rgb8,
                BitDepth::B16 => ColorType::Rgb16,
                BitDepth::B32 => ColorType::Rgb32F,
            },
            ColorSpace::RgbA => match self.bit_depth {
                BitDepth::B8 => ColorType::Rgba8,
                BitDepth::B16 => ColorType::Rgba16,
                BitDepth::B32 => ColorType::Rgba32F,
            },
            ColorSpace::Luma => match self.bit_depth {
                BitDepth::B8 => ColorType::L8,
                BitDepth::B16 => ColorType::L16,
                BitDepth::B32 => ColorType::L16,
            },
            ColorSpace::LumaA => match self.bit_depth {
                BitDepth::B8 => ColorType::La8,
                BitDepth::B16 => ColorType::La16,
                BitDepth::B32 => ColorType::La16,
            },
            ColorSpace::Unknown => ColorType::Rgb8,
        }
    }
}
impl From<ColorType> for ColorInfo {
    fn from(value: ColorType) -> Self {
        match value {
            ColorType::L8 => Self {
                bit_depth: BitDepth::B8,
                color_space: ColorSpace::Luma,
            },
            ColorType::La8 => Self {
                bit_depth: BitDepth::B8,
                color_space: ColorSpace::LumaA,
            },
            ColorType::Rgb8 => Self {
                bit_depth: BitDepth::B8,
                color_space: ColorSpace::Rgb,
            },
            ColorType::Rgba8 => Self {
                bit_depth: BitDepth::B8,
                color_space: ColorSpace::RgbA,
            },
            ColorType::L16 => Self {
                bit_depth: BitDepth::B16,
                color_space: ColorSpace::Luma,
            },
            ColorType::La16 => Self {
                bit_depth: BitDepth::B16,
                color_space: ColorSpace::LumaA,
            },
            ColorType::Rgb16 => Self {
                bit_depth: BitDepth::B16,
                color_space: ColorSpace::Rgb,
            },
            ColorType::Rgba16 => Self {
                bit_depth: BitDepth::B16,
                color_space: ColorSpace::RgbA,
            },
            ColorType::Rgb32F => Self {
                bit_depth: BitDepth::B32,
                color_space: ColorSpace::Rgb,
            },
            ColorType::Rgba32F => Self {
                bit_depth: BitDepth::B32,
                color_space: ColorSpace::RgbA,
            },
            _ => Self::default(),
        }
    }
}
