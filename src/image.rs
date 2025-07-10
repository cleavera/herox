use xcap::image::{Rgba, RgbaImage};

#[napi(object)]
pub struct Pixel {
    pub x: u32,
    pub y: u32,
}

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

  #[napi]
  pub fn find_rgbas(&self, rgba_number: u32) -> Vec<Pixel> {
    let r = ((rgba_number >> 24) & 0xFFu32) as u8;
    let g = ((rgba_number >> 16) & 0xFFu32) as u8;
    let b = ((rgba_number >> 8) & 0xFFu32) as u8;
    let a = (rgba_number & 0xFFu32) as u8;

    let rgba = Rgba([r, g, b, a]);

    let mut positions = Vec::new();

    for (x, y, pixel) in self.rgba_image.enumerate_pixels() {
      if *pixel == rgba {
        positions.push(Pixel { x, y });
      }
    }

    positions
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
