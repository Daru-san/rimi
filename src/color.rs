use std::str::FromStr;

use image::{ColorType, DynamicImage};

#[derive(Default, Debug, Clone)]
pub struct ColorInfo {
    pub bit_depth: u32,
    pub color_type: String,
    pub is_alpha: bool,
}

impl ColorInfo {
    pub fn from_image(image: &DynamicImage) -> Self {
        match image.color() {
            ColorType::L8 => ColorInfo {
                bit_depth: 8,
                color_type: "Luminant".to_string(),
                is_alpha: false,
            },
            ColorType::La8 => ColorInfo {
                bit_depth: 8,
                color_type: "Luminant".to_string(),
                is_alpha: true,
            },
            ColorType::L16 => ColorInfo {
                bit_depth: 16,
                color_type: "Luminant".to_string(),
                is_alpha: true,
            },
            ColorType::La16 => ColorInfo {
                bit_depth: 16,
                color_type: "Luminant".to_string(),
                is_alpha: true,
            },
            ColorType::Rgb8 => ColorInfo {
                bit_depth: 8,
                color_type: "RGB".to_string(),
                is_alpha: false,
            },
            ColorType::Rgba8 => ColorInfo {
                bit_depth: 8,
                color_type: "RGB".to_string(),
                is_alpha: true,
            },
            ColorType::Rgb16 => ColorInfo {
                bit_depth: 16,
                color_type: "RGB".to_string(),
                is_alpha: false,
            },
            ColorType::Rgba16 => ColorInfo {
                bit_depth: 16,
                color_type: "RGB".to_string(),
                is_alpha: true,
            },
            ColorType::Rgb32F => ColorInfo {
                bit_depth: 32,
                color_type: "RGB".to_string(),
                is_alpha: false,
            },
            ColorType::Rgba32F => ColorInfo {
                bit_depth: 32,
                color_type: "RGB".to_string(),
                is_alpha: true,
            },
            _ => ColorInfo {
                bit_depth: 8,
                color_type: "Unknown".to_string(),
                is_alpha: false,
            },
        }
    }
    pub fn convert_image(&self, image: &mut DynamicImage) {
        let color_type = self.get_color_type();
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
    fn get_color_type(&self) -> ColorType {
        match self.color_type.as_str() {
            "RGB" => match self.bit_depth {
                8 => {
                    if self.is_alpha {
                        ColorType::Rgba8
                    } else {
                        ColorType::Rgb8
                    }
                }
                16 => {
                    if self.is_alpha {
                        ColorType::Rgba16
                    } else {
                        ColorType::Rgb16
                    }
                }
                32 => {
                    if self.is_alpha {
                        ColorType::Rgba32F
                    } else {
                        ColorType::Rgb32F
                    }
                }
                _ => ColorType::Rgb8,
            },
            "Luminant" => match self.bit_depth {
                8 => {
                    if self.is_alpha {
                        ColorType::La8
                    } else {
                        ColorType::L8
                    }
                }
                16 => {
                    if self.is_alpha {
                        ColorType::La16
                    } else {
                        ColorType::L16
                    }
                }
                _ => ColorType::L8,
            },
            _ => ColorType::Rgb8,
        }
    }
}

impl FromStr for ColorInfo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let col_type;
        let bit_depth;
        let is_alpha = s.contains("alpha");

        if s.contains("luma") {
            col_type = "Luminant";
        } else if s.to_lowercase().contains("rgb") {
            col_type = "RGB";
        } else {
            col_type = "Unknown";
        }
        if s.contains("8") {
            bit_depth = 8;
        } else if s.contains("16") {
            bit_depth = 16
        } else if s.contains("32") {
            bit_depth = 32;
        } else {
            bit_depth = 0;
        }

        Ok(ColorInfo {
            color_type: col_type.to_string(),
            bit_depth,
            is_alpha,
        })
    }
}
