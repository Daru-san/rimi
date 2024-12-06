mod batch;
mod color;
mod info;
mod open;
mod utils;

use std::path::PathBuf;

use batch::*;
use clap::{Parser, Subcommand};
use color::ColorInfo;
use image::{ColorType, ImageReader};
use info::*;
use open::open_image;
use utils::*;

/// Simple image manipulation tool
#[derive(Parser, Debug)]
#[command(version,about,long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Option<Commands>,

    /// Overwrite any existing files when saving the image
    #[clap(short = 'x', long, default_value = "false", global = true)]
    overwrite: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert an image
    #[clap(short_flag = 'c')]
    Convert {
        /// Format of the new image.
        #[clap(short, long)]
        format: Option<String>,

        /// Path to the input image
        image_file: PathBuf,

        /// Path where the saved image should be written
        #[clap(short, long, global = true)]
        output: Option<String>,
    },

    /// Resize an image
    #[clap(short_flag = 'r')]
    Resize {
        /// Path to the image file
        image_file: PathBuf,

        /// Path where the saved image should be written
        #[clap(short, long, global = true)]
        output: Option<String>,

        /// New width
        width: u32,

        /// New height
        height: u32,

        /// Image Sampling filter
        #[clap(short, long, default_value = "Nearest")]
        filter: String,

        /// Preserve aspect ratio
        #[clap(short, long)]
        preserve_aspect: bool,
    },

    /// Show image information
    #[clap(short_flag = 'i')]
    Info {
        #[clap(short, long)]
        short: bool,

        ///Path to the image file
        image_file: PathBuf,
    },

    /// Batch image conversion
    #[clap(short_flag = 'b')]
    Batch {
        /// Images to be converted
        #[clap(value_parser,num_args = 1..100,value_delimiter = ' ',required = true)]
        images: Vec<String>,

        /// Optional output directory where all output images will be saved
        #[clap(short, long, default_value = ".")]
        directory: String,

        /// Expression that the output image names will follow
        #[clap(short, long)]
        name_expr: Option<String>,
    },

    /// Remove the background from an image
    #[clap(short_flag = 't')]
    Transparentize {
        /// Path to the input image
        image_file: PathBuf,

        /// Path where the saved image should be written
        #[clap(short, long, global = true)]
        output: Option<String>,
    },

    Recolor {
        /// Path to the input image
        image_file: PathBuf,

        /// Path where the saved image should be written
        #[clap(short, long, global = true)]
        output: Option<String>,

        /// Color type
        #[clap(short, long)]
        color_type: ColorInfo,
    },
}

fn main() {
    let args = Args::parse();

    let do_overwrite = &args.overwrite;

    match &args.cmd {
        Some(Commands::Convert {
            format,
            image_file,
            output,
        }) => {
            let image = open_image(image_file.into());

            let output_path = match output {
                Some(e) => e.as_str(),
                None => image_file
                    .as_os_str()
                    .to_str()
                    .expect("Error parsing image path: "),
            };
            save_image_format(&image, output_path, format.as_deref(), *do_overwrite);
        }
        Some(Commands::Resize {
            width,
            height,
            filter,
            preserve_aspect,
            image_file,
            output,
        }) => {
            let mut image = open_image(image_file.into());

            let output_path = match output {
                Some(e) => e.as_str(),
                None => image_file
                    .as_os_str()
                    .to_str()
                    .expect("Error parsing image path: "),
            };
            resize_image(
                &mut image,
                Dimensions {
                    x: *width,
                    y: *height,
                },
                filter.to_string(),
                *preserve_aspect,
            );
            save_image_format(&image, output_path, None, *do_overwrite);
        }
        Some(Commands::Info { short, image_file }) => {
            let image = open_image(image_file.into());
            print_info(&image, image_file.to_path_buf(), *short);
        }
        Some(Commands::Batch {
            images,
            directory,
            name_expr,
        }) => {
            let images_str: Vec<&str> = images.iter().map(|s| s.as_str()).collect();
            check_batch(images_str.clone());
            let paths = create_paths(images_str.clone(), directory.as_str(), name_expr.as_deref());

            let mut i = 0;
            #[allow(clippy::explicit_counter_loop)]
            for image_str in images {
                let image = ImageReader::open(image_str).unwrap().decode().unwrap();
                save_image_format(&image, &paths[i], None, *do_overwrite);
                i += 1;
            }
        }
        Some(Commands::Transparentize { image_file, output }) => {
            use image::ImageFormat;
            use std::process::exit;

            let mut image = open_image(image_file.into());

            let output_path = match output {
                Some(e) => e.as_str(),
                None => image_file
                    .as_os_str()
                    .to_str()
                    .expect("Error parsing image path: "),
            };
            if ImageFormat::from_path(image_file).unwrap() != ImageFormat::Png {
                eprintln!("{:?}: Image must be a png file.", image_file.as_os_str());
                exit(0);
            }
            remove_background(&mut image);
            save_image_format(&image, output_path, None, *do_overwrite);
        }
        Some(Commands::Recolor {
            image_file,
            output,
            color_type,
        }) => {
            let mut image = open_image(image_file.into());

            let output_path = match output {
                Some(e) => e.as_str(),
                None => image_file
                    .as_os_str()
                    .to_str()
                    .expect("Error parsing image path: "),
            };
            color_type.convert_image(&mut image);
            save_image_format(&image, output_path, None, *do_overwrite)
        }
        None => {
            println!("Please select one of: resize or convert.");
            println!("Use -h to get usage.")
        }
    }
}
