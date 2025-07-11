use crate::image::Image;
use napi::bindgen_prelude::{AsyncTask, Env, Error, Task};
use xcap::{Window as XCapWindow, XCapError};

pub struct WindowError {
  message: String,
}

impl From<XCapError> for WindowError {
  fn from(value: XCapError) -> Self {
    WindowError {
      message: value.to_string(),
    }
  }
}

impl From<WindowError> for Error {
  fn from(value: WindowError) -> Error {
    Error::from_reason(value.message)
  }
}

#[napi]
#[derive(Debug, Clone)]
pub struct Window {
  x_cap_window: XCapWindow,
}

#[napi]
impl Window {
  fn new(x_cap_window: &XCapWindow) -> Self {
    Window {
      x_cap_window: x_cap_window.clone(),
    }
  }

  #[napi]
  pub fn all() -> Result<Vec<Window>, Error> {
    let monitors = XCapWindow::all()
      .map_err(WindowError::from)?
      .iter()
      .map(Window::new)
      .collect();

    Ok(monitors)
  }
}

#[napi]
impl Window {
  #[napi]
  pub fn title(&self) -> Result<String, Error> {
    Ok(self.x_cap_window.title().map_err(WindowError::from)?)
  }

  #[napi]
  pub fn x(&self) -> Result<i32, Error> {
    Ok(self.x_cap_window.x().map_err(WindowError::from)?)
  }

  #[napi]
  pub fn y(&self) -> Result<i32, Error> {
    Ok(self.x_cap_window.y().map_err(WindowError::from)?)
  }

  #[napi]
  pub fn z(&self) -> Result<i32, Error> {
    Ok(self.x_cap_window.z().map_err(WindowError::from)?)
  }

  #[napi]
  pub fn width(&self) -> Result<u32, Error> {
    Ok(self.x_cap_window.width().map_err(WindowError::from)?)
  }

  #[napi]
  pub fn height(&self) -> Result<u32, Error> {
    Ok(self.x_cap_window.height().map_err(WindowError::from)?)
  }

  #[napi]
  pub fn is_focused(&self) -> Result<bool, Error> {
    Ok(self.x_cap_window.is_focused().map_err(WindowError::from)?)
  }

  #[napi(ts_return_type = "Promise<Image>")]
  pub fn capture_image(&self) -> AsyncTask<AsyncCaptureImage> {
    AsyncTask::new(AsyncCaptureImage::new(self.x_cap_window.clone()))
  }
}

pub struct AsyncCaptureImage {
  window: XCapWindow,
}

impl AsyncCaptureImage {
  pub fn new(window: XCapWindow) -> Self {
    Self { window }
  }
}

#[napi]
impl Task for AsyncCaptureImage {
  type Output = Image;
  type JsValue = Image;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let rgba_image = self.window.capture_image().map_err(WindowError::from)?;

    Ok(Image::from(rgba_image))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue, Error> {
    Ok(output)
  }
}
