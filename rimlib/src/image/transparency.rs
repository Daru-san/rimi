use super::color::ColorInfo;
use image::DynamicImage;
use rayon::iter::ParallelBridge;

pub fn remove_background(image: DynamicImage) -> DynamicImage {
    use super::color::BitDepth::{B8, B16, B32};
    let color_info = ColorInfo::from_image(&image);

    match color_info.bit_depth {
        B8 => {
            let mut image8bit = image.to_rgba8();
            image8bit.pixels_mut().par_bridge().for_each(|pixel| {
                if pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 255 {
                    pixel[3] = 0;
                }
            });
            DynamicImage::ImageRgba8(image8bit)
        }

        B16 => {
            let mut image16bit = image.to_rgba16();
            image16bit.pixels_mut().par_bridge().for_each(|pixel| {
                if pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 255 {
                    pixel[3] = 0;
                }
            });
            DynamicImage::ImageRgba16(image16bit)
        }

        B32 => {
            let mut image32bit = image.to_rgba32f();
            image32bit.pixels_mut().par_bridge().for_each(|pixel| {
                if pixel[0] == 255.0 && pixel[1] == 255.0 && pixel[2] == 255.0 {
                    pixel[3] = 0.0;
                }
            });
            DynamicImage::ImageRgba32F(image32bit)
        }
    }
}
