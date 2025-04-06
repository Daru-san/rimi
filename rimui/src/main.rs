use gallery::ImageGallery;
mod gallery;
mod imagedef;
mod widgets;

#[tokio::main]
async fn main()  -> Result<(), iced::Error> {
    iced::run("Hello!",ImageGallery::update, ImageGallery::view)
}
