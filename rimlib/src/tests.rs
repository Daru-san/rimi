use std::{
    io::{BufWriter, Cursor},
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use crate::image::randomize::Randomizer;
use ::image::DynamicImage;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[test]
fn resize() {
    let mut images = Vec::<DynamicImage>::new();

    for _ in 1..10 {
        let image = DynamicImage::new_luma_a16(1920, 1080);

        let image = match image.randomize_size(480, 270, 3840, 2160, None) {
            Ok(resulting) => resulting,
            Err(e) => panic!("{e:?}"),
        };

        images.push(image);
    }
}

#[test]
fn mass_write() {
    let mut images: Vec<DynamicImage> = Vec::new();

    for _ in 1..10 {
        let image = DynamicImage::new_luma_a16(1920, 1080);

        let image = match image.randomize_all() {
            Ok(resulting) => resulting,
            Err(e) => panic!("{e:?}"),
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
                        let dur = Duration::from(end);
                        tx.send(format!(
                            r#"
                                Image written successfully!
                                Took {} seconds
                            "#,
                            dur.as_secs()
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
