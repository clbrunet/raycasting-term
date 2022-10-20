use image::io::Reader as ImageReader;
use std::io::{BufRead, Seek};

use image::DynamicImage;

pub enum SpriteSheetKind {
    Single,
    EightDirectional,
}

pub struct SpriteSheet {
    image: DynamicImage,
    kind: SpriteSheetKind,
    size: u32,
}

impl SpriteSheet {
    pub fn new<R: BufRead + Seek>(buffered_reader: R) -> Self {
        let image = ImageReader::new(buffered_reader)
            .with_guessed_format()
            .expect("Image reading error")
            .decode()
            .expect("Image decoding error");
        assert_eq!(image.height() % image.width(), 0);
        let kind = match image.height() / image.width() {
            1 => SpriteSheetKind::Single,
            8 => SpriteSheetKind::EightDirectional,
            _ => panic!(
                "Image width and height ratio not supported, width: {}, height: {}",
                image.width(),
                image.height()
            ),
        };
        let size = image.width();
        Self { image, kind, size }
    }

    pub fn image(&self) -> &DynamicImage {
        &self.image
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn get_y_offset(&self, angle: f64) -> u32 {
        let degrees = angle.to_degrees();
        match &self.kind {
            SpriteSheetKind::Single => 0,
            SpriteSheetKind::EightDirectional => {
                if degrees < 22.5 || 360.5 < degrees {
                    0
                } else if degrees < 67.5 {
                    7 * self.size
                } else if degrees < 112.5 {
                    6 * self.size
                } else if degrees < 157.5 {
                    5 * self.size
                } else if degrees < 202.5 {
                    4 * self.size
                } else if degrees < 247.5 {
                    3 * self.size
                } else if degrees < 292.5 {
                    2 * self.size
                } else {
                    self.size
                }
            },
        }
    }
}
