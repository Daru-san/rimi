use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Cursor};

use iced::{Alignment, Element, Length};
use iced_widget::image::Handle;
use iced_widget::{Column, Container, button, keyed, keyed_column, text};
use image::{DynamicImage, GenericImageView, ImageReader};

/// Structure containing image view data
/// Currently only support DynamicImage values
#[derive(Default, Debug, Clone)]
struct ImageView {
    data: Box<DynamicImage>,
    width: u32,
    height: u32,
}

/// Image gallery contains a hashmap of image views
/// Each image view has an integer key that must be unique
struct ImageGallery {
    views: HashMap<u32, ImageView>,
}

impl PartialEq for ImageView {
    fn eq(&self, other: &Self) -> bool {
        (self.width == other.width) == (self.height == other.height)
            && self
                .data
                .clone()
                .into_bytes()
                .iter()
                .zip(other.data.as_bytes())
                .all(|(a, b)| a == b)
    }
}

/// Messages that can be sent to the image gallery
#[derive(Debug, Clone, PartialEq, Copy)]
enum ImageGalleryMessage {
    /// Insert a new image into the gallery
    /// Must be sent with a key and an ImageView
    Insert(u32),

    /// Delete an image from the gallery
    Delete(u32),
}

impl ImageGallery {
    /// Create a new instance of the image gallery with empty views
    fn new() -> ImageGallery {
        ImageGallery {
            views: HashMap::new(),
        }
    }

    /// Send an update request to the image gallery.
    ///
    /// Messages can insert images or delete images from the gallery
    fn update(&mut self, message: ImageGalleryMessage) {
        match message {
            ImageGalleryMessage::Insert(id) => {}
            ImageGalleryMessage::Delete(id) => {
                self.views.remove(&id);
            }
        }
    }

    fn push_image<B>(&mut self, data: B)
    where
        B: Into<Vec<u8>>,
    {
        let bufreader = BufReader::new(Cursor::new(data.into()));
        let reader = ImageReader::new(bufreader).with_guessed_format().unwrap();
        let image = reader.decode().unwrap();
        let prev_idx = self.views.iter().last().unwrap().0;
        self.views.insert(
            prev_idx.wrapping_add(1),
            ImageView {
                data: Box::new(image.clone()),
                width: image.width(),
                height: image.height(),
            },
        );
    }

    fn view(&self) -> Element<'_, ImageGalleryMessage> {
        keyed_column(self.views.iter().map(|(i, v)| {
            let delete = button("Delete").on_press(ImageGalleryMessage::Delete(*i));
            let text: Element<_> = text!("Image: {}x{}", v.width, v.height).into();

            let image = iced_widget::Image::new(Handle::from_bytes(v.data.clone().into_bytes()))
                .filter_method(iced_widget::image::FilterMethod::Nearest)
                .width(Length::Fill)
                .height(Length::Fill);

            let image_container = Container::new(image)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill);

            let column = Column::new()
                .padding(20)
                .push(image_container)
                .push(text)
                .push(delete);

            (*i, column.into())
        }))
        .into()
    }
}

#[test]
fn gallery_messages() {
    let mut gallery = ImageGallery::new();

    for i in 1..10 {
        let imgview = (i, ImageView::default());
        gallery.update(ImageGalleryMessage::Insert(imgview.0));
        gallery.push_image(imgview.1.data.into_bytes());
    }

    gallery.update(ImageGalleryMessage::Delete(3));
    gallery.update(ImageGalleryMessage::Delete(4));
    gallery.update(ImageGalleryMessage::Delete(5));

    assert!(gallery.views.contains_key(&1));
    assert!(gallery.views.contains_key(&2));

    assert!(!gallery.views.contains_key(&3));
    assert!(!gallery.views.contains_key(&4));
    assert!(!gallery.views.contains_key(&5));

    assert!(!gallery.views.contains_key(&12));

    for i in 6..10 {
        assert!(gallery.views.contains_key(&i));
    }
}
