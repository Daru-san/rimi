use iced_widget::core::image::Bytes;
use image::{DynamicImage, ImageReader};
use rfd::AsyncFileDialog;
use std::io::{Cursor, ErrorKind};
use std::path::{Path, PathBuf};
use tokio::fs::{self, read_dir};

#[derive(Debug, Clone)]
pub struct GalleryImage {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Bytes,
    pub(crate) path: PathBuf,
}

impl GalleryImage {
    pub async fn read_image(path: impl AsRef<Path>) -> Result<GalleryImage, ImageGalleryError> {
        let raw = fs::read(path)
            .await
            .map_err(|err| err.kind())
            .map_err(ImageGalleryError::IO)?;
        let cursor = Cursor::new(raw);
        ImageReader::new(cursor)
            .with_guessed_format()
            .map_err(|_| ImageGalleryError::ReadFailed(0))?
            .decode()
            .map_err(|e| ImageGalleryError::Image(e.to_string()))
            .map(GalleryImage::from)
    }

    pub async fn read_multiple(
        paths: Vec<&Path>,
    ) -> Result<Vec<Result<GalleryImage, ImageGalleryError>>, ImageGalleryError> {
        let mut handles = Vec::new();

        for path in paths {
            let job = tokio::spawn(Self::read_image(path.to_path_buf()));
            handles.push(job);
        }

        let mut images = Vec::new();

        for job in handles {
            images.push(job.await.unwrap_or(Err(ImageGalleryError::ReadFailed(0))));
        }
        Ok(images)
    }

    pub async fn pick_multiple()
    -> Result<Vec<Result<GalleryImage, ImageGalleryError>>, ImageGalleryError> {
        let dir_handle = AsyncFileDialog::new()
            .set_title("Pick a directory")
            .pick_folder()
            .await
            .ok_or(ImageGalleryError::DialogClosed)?;

        let path = dir_handle.path();

        let mut reader = read_dir(path)
            .await
            .map_err(|e| e.kind())
            .map_err(ImageGalleryError::IO)?;

        let mut images = Vec::new();

        while let Some(entry) = reader
            .next_entry()
            .await
            .map_err(|e| ImageGalleryError::Image(e.to_string()))?
        {
            images.push(Self::read_image(entry.path()).await)
        }
        Ok(images)
    }

    pub async fn pick_single() -> Result<GalleryImage, ImageGalleryError> {
        let fhandle = AsyncFileDialog::new()
            .set_title("Choose a text file")
            .pick_file()
            .await
            .ok_or(ImageGalleryError::DialogClosed)?;

        let path = PathBuf::from(fhandle.path());
        Self::read_image(path).await
    }
}

impl From<DynamicImage> for GalleryImage {
    fn from(value: DynamicImage) -> Self {
        let width = value.width();
        let height = value.height();
        let data = value.into_bytes();
        let bytes = Bytes::from(data);
        GalleryImage {
            width,
            height,
            pixels: bytes,
            path: PathBuf::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ImageGalleryError {
    ReadFailed(u8),
    IO(ErrorKind),
    Image(String),
    DialogClosed,
}
