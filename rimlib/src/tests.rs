use std::{
    io::{BufWriter, Cursor},
    sync::mpsc::channel,
    time::Instant,
};

use crate::image::transparency::Transparenize;
use crate::image::randomize::Randomizer;
use image::{DynamicImage, ColorType};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[test]
fn resize() {
    let mut images = Vec::<DynamicImage>::new();

    for _ in 1..10 {
        let image = {
            let i = DynamicImage::new_luma_a16(1920, 1080);

            i.randomize_size(480, 270, 3840, 2160, None)
        };


        images.push(image);
    }
}

#[test]
fn mass_write() {
    let mut images: Vec<DynamicImage> = Vec::new();

    for _ in 1..10 {
        let image = {
            let i = DynamicImage::new_luma_a16(1920, 1080);

            i.randomize_all()
        };


        images.push(image);
    }

    let (tx, rx) = channel();
    rayon::scope(move |s| {
        let tx = tx.clone();

        s.spawn(move |_| {
            images.par_iter().for_each_with(tx, |tx, image| {
                let mut writer = BufWriter::new(Cursor::new(Vec::new()));
                let begin = Instant::now();
                match image.write_to(&mut writer, ::image::ImageFormat::Avif) {
                    Ok(_) => {
                        let end = Instant::now().duration_since(begin);
                        tx.send(format!(
                            r#"
                                Image written successfully!
                                Took {} seconds
                            "#,
                            end.as_secs()
                        ))
                        .unwrap_or_default();
                    }
                    Err(e) => {
                        tx.send(format!("Image failed to write with error.\n{e:?}"))
                            .unwrap_or_default();
                    }
                };
            });
        });

        rx.iter().fuse().for_each(|result| {
            println!("{result:?}");
        });
    });
}

#[test]
fn randomize_test() {
    let mut images: Vec<DynamicImage> = Vec::new();

    for _ in 1..10 {
        let image = {
            let i = DynamicImage::new_luma_a16(1920, 1080);

            i.randomize_all()
        };

        images.push(image);
    }
}

#[test]
fn multi_random() {
    let mut images: Vec<DynamicImage> = Vec::new();

    for _ in 1..10 {
        let image = {
            let i = DynamicImage::new_luma_a16(1920, 1080);

            i.randomize_hue().randomize_size(640, 360, 15360, 8640, None)
        };

        images.push(image);
    }
}

#[test]
fn transparency() {
    let mut images = Vec::<DynamicImage>::new();

    for _ in 1..10 {
        let image = {
            let i = DynamicImage::new_rgba16(1920, 1080);

            i.randomize_color(ColorType::Rgba16).transparentize()
        };

        images.push(image);
    }
}
