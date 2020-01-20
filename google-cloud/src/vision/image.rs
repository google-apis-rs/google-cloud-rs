use std::io;

use crate::vision::api;

/// Represents an image, for use in Cloud Vision APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    // inner enum to not leak internal variants (enums variants are all public).
    // this makes `Image` act like an opaque enum.
    pub(crate) inner: ImageInner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ImageInner {
    Bytes(Vec<u8>),
    URL(String),
}

impl Image {
    /// Constructs an image directly from bytes.
    pub fn from_bytes(data: impl Into<Vec<u8>>) -> Image {
        Image {
            inner: ImageInner::Bytes(data.into()),
        }
    }

    /// Constructs an image from URL.
    pub fn from_url(url: impl Into<String>) -> Image {
        Image {
            inner: ImageInner::URL(url.into()),
        }
    }

    /// Constructs an image by pulling the bytes from an IO reader.
    pub fn from_reader(mut rdr: impl io::Read) -> io::Result<Image> {
        let mut data = Vec::new();
        rdr.read_to_end(&mut data)?;
        Ok(Image::from_bytes(data))
    }
}

impl From<Image> for api::Image {
    fn from(img: Image) -> api::Image {
        match img.inner {
            ImageInner::Bytes(content) => api::Image {
                content,
                source: None,
            },
            ImageInner::URL(image_uri) => api::Image {
                content: Vec::new(),
                source: Some(api::ImageSource {
                    image_uri,
                    gcs_image_uri: String::default(),
                }),
            },
        }
    }
}
