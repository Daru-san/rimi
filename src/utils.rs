
pub fn save_image(image: &DynamicImage, path: &str) {
    image.save(path).expect("File save error:");
}
