use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use image::{self, imageops, ImageFormat};
use std::path::Path;
use std::process::exit;

#[derive(Debug, Default)]
pub struct Image {
    pub filename: String,
    pub image: image::DynamicImage,
    pub format: String,
    pub outpath: String,
}

impl Image {
    pub fn new(fname: String) -> Self {
        let img = image::open(&fname).unwrap();
        Image {
            filename: fname.clone(),
            image: img,
            format: "default".to_string(),
            outpath: fname.clone(),
        }
    }
    pub fn convert(&mut self) {
        if self.format != "default" {
            self.convert_raw();
        } else {
            self.convert_format();
        }
    }
    fn convert_raw(&mut self) {
        self.image
            .save(&self.outpath)
            .expect("Error converting image");
    }
    fn convert_format(&mut self) {
        #[allow(clippy::needless_late_init)]
        let img_format;

        match self.format.to_uppercase().as_str() {
            "PNG" => img_format = ImageFormat::Png,
            "JPG" => img_format = ImageFormat::Jpeg,
            "JPEG" => img_format = ImageFormat::Jpeg,
            "WEBP" => img_format = ImageFormat::WebP,
            "ICO" => img_format = ImageFormat::Ico,
            "GIF" => img_format = ImageFormat::Gif,
            "AVIF" => img_format = ImageFormat::Avif,
            _ => {
                println!("Please enter a format");
                exit(12);
            }
        };

        self.check_overwrite();

        self.image
            .save_with_format(&self.filename, img_format)
            .expect("Error converting image");
    }

    pub fn resize_dim(&mut self, w: u32, h: u32) {
        self.image
            .resize(w, h, image::imageops::FilterType::Nearest);
    }

    pub fn resize_scale(&mut self, scale: f32) {
        let w = (self.image.width() as f32 * scale).trunc() as u32;
        let h = (self.image.height() as f32 * scale).trunc() as u32;
        self.image.resize(w, h, imageops::FilterType::Nearest);
    }

    fn check_overwrite(&mut self) {
        let img_path = Path::new(&self.filename);
        if img_path.exists() && self.filename == self.outpath {
            self.prompt_overwrite("Overwrite file?");
        }
    }

    fn prompt_overwrite(&self, msg: &str) {
        let confirmation = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(msg)
            .default(true)
            .interact()
            .unwrap();
        if !confirmation {
            exit(0);
        }
    }
}
