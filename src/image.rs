use xcap::image::RgbaImage;

#[napi]
#[derive(Debug, Clone)]
pub struct Image {
    rgba_image: RgbaImage,
    #[napi(readonly)]
    pub width: u32,
    #[napi(readonly)]
    pub height: u32,
}

#[napi]
impl Image {
    #[napi]
    pub fn get_pixel_rgba(&self, x: u32, y: u32) -> u32 {
        let rgba_pixel: &Rgba<u8> = self.rgba_image.get_pixel(x, y);

        ((rgba_pixel.0[0] as u32) << 24)
            | ((rgba_pixel.0[1] as u32) << 16)
            | ((rgba_pixel.0[2] as u32) << 8)
            | (rgba_pixel.0[3] as u32)
    }
}

impl From<RgbaImage> for Image {
    fn from(value: RgbaImage) -> Self {
        Image {
            width: value.width(),
            height: value.height(),
            rgba_image: value,
        }
    }
}

