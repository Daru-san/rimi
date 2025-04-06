use core::error;
use std::collections::HashMap;
use std::io;

use ::image::ImageError;
use iced::{Element, Length, Task, Theme};
use iced_widget::core::image::Bytes;
use iced_widget::image::Handle;
use iced_widget::{
    Button, Column, Container, Image, button, center, column, container, horizontal_space, image,
    keyed_column, mouse_area, opaque, row, scrollable, stack, text,
};

pub enum Error {
    IO(io::ErrorKind),
    Image(ImageError),
    Unknown,
}

use crate::imagedef::{GalleryImage, ImageGalleryError};

#[derive(Debug, Clone)]
pub enum Message {
    ImagesRead(Result<Vec<Result<GalleryImage, ImageGalleryError>>, ImageGalleryError>),
    OpenImages,
}

#[derive(Default)]
pub struct ImageGallery {
    images: Vec<GalleryImage>,
    views: HashMap<u32, ImageView>,
}

struct ImageView {}

impl ImageGallery {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                images: Vec::new(),
                views: HashMap::new(),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImagesRead(Ok(images)) => {
                self.images = images.iter().flatten().cloned().collect();

                Task::none()
            }
            Message::OpenImages => {
                Task::perform(GalleryImage::pick_multiple(), Message::ImagesRead)
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let gallery = if self.images.is_empty() {
            let button: Button<'_, Message, Theme> =
                Button::new("Open images in a directory").on_press(Message::OpenImages);
            let column = column!["No images are open", button];
            row![
                container(column)
                    .center_x(Length::Shrink)
                    .center_y(Length::Shrink)
            ]
        } else {
            row(self
                .images
                .iter()
                .map(|image| {
                    let image: Element<'_, Message> = Image::new(Handle::from_rgba(
                        image.width,
                        image.height,
                        image.pixels.clone(),
                    ))
                    .into();
                    container(image).width(320).height(410).into()
                })
                .collect::<Vec<Element<'_, Message>>>())
        }
        .spacing(10)
        .wrap();

        let content: Element<'_, Message> = container(center(gallery))
            .padding(10)
            .into();

        stack![content].into()
    }
}

//
// #[test]
// fn gallery_messages() {
//     let mut gallery = ImageGallery::new(None);
//
//     for i in 1..10 {
//         let imgview = (i, ImageView::default());
//         gallery.update(ImageGalleryMessage::Insert(imgview.0));
//     }
//
//     gallery.update(ImageGalleryMessage::Delete(3));
//     gallery.update(ImageGalleryMessage::Delete(4));
//     gallery.update(ImageGalleryMessage::Delete(5));
//
//     assert!(gallery.views.contains_key(&1));
//     assert!(gallery.views.contains_key(&2));
//
//     assert!(!gallery.views.contains_key(&3));
//     assert!(!gallery.views.contains_key(&4));
//     assert!(!gallery.views.contains_key(&5));
//
//     assert!(!gallery.views.contains_key(&12));
//
//     for i in 6..10 {
//         assert!(gallery.views.contains_key(&i));
//     }
// }
