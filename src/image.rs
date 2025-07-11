use napi::{bindgen_prelude::AsyncTask, Env, Error, JsNumber, Task};
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
  #[napi(ts_return_type="Promise<number>")]
  pub fn get_pixel_rgba(&self, x: u32, y: u32) -> AsyncTask<AsyncGetPixelRgba> {
    AsyncTask::new(AsyncGetPixelRgba::new(x, y, self.rgba_image.clone()))
  }

  #[napi(ts_return_type="Promise<Array<Pixel>>")]
  pub fn find_rgbas(&self, rgba_number: u32) -> AsyncTask<AsyncFindRgbas> {
    AsyncTask::new(AsyncFindRgbas::new(rgba_number, self.rgba_image.clone()))
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

pub struct AsyncGetPixelRgba {
  x: u32,
  y: u32,
  rgba_image: RgbaImage,
}

impl AsyncGetPixelRgba {
  pub fn new(x: u32, y: u32, rgba_image: RgbaImage) -> Self {
    Self {
      x,
      y,
      rgba_image,
    }
  }
}

#[napi]
impl Task for AsyncGetPixelRgba {
  type Output = u32;
  type JsValue = JsNumber;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let rgba_pixel: &Rgba<u8> = self.rgba_image.get_pixel_checked(self.x, self.y).ok_or(Error::from_reason("Pixel out of bounds"))?;

    Ok(((rgba_pixel.0[0] as u32) << 24)
      | ((rgba_pixel.0[1] as u32) << 16)
      | ((rgba_pixel.0[2] as u32) << 8)
      | (rgba_pixel.0[3] as u32))
  }

  fn resolve(&mut self, env: Env, output: Self::Output) -> Result<Self::JsValue, Error> {
    Ok(env.create_uint32(output as _)?)
  }
}

pub struct AsyncFindRgbas {
  rgba_number: u32,
  rgba_image: RgbaImage,
}

impl AsyncFindRgbas {
  pub fn new(rgba_number: u32, rgba_image: RgbaImage) -> Self {
    Self {
      rgba_number,
      rgba_image,
    }
  }
}

#[napi]
impl Task for AsyncFindRgbas {
  type Output = Vec<Pixel>;
  type JsValue = Vec<Pixel>;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let r = ((self.rgba_number >> 24) & 0xFFu32) as u8;
    let g = ((self.rgba_number >> 16) & 0xFFu32) as u8;
    let b = ((self.rgba_number >> 8) & 0xFFu32) as u8;
    let a = (self.rgba_number & 0xFFu32) as u8;

    let rgba = Rgba([r, g, b, a]);

    let mut positions = Vec::new();

    for (x, y, pixel) in self.rgba_image.enumerate_pixels() {
      if *pixel == rgba {
        positions.push(Pixel { x, y });
      }
    }

    Ok(positions)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue, Error> {
    Ok(output)
  }
}
