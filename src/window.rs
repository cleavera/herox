use napi::{bindgen_prelude::AsyncTask, Env, Error, Task};

use crate::image::Image;

#[cfg(target_os = "windows")]
mod windows_backend;
#[cfg(target_os = "windows")]
use windows_backend::WindowsWindow;

#[cfg(target_os = "linux")]
mod x11_backend;
#[cfg(target_os = "linux")]
use x11_backend::X11Window;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod unsupported_backend;

#[derive(Debug)]
pub enum WindowError {
  ApiError(String),
  EmptyTitle,
  InvalidBitmap,
  UnsupportedPlatform,
}

impl From<WindowError> for Error {
  fn from(value: WindowError) -> Error {
    let message = match value {
      WindowError::ApiError(s) => s,
      WindowError::EmptyTitle => "Window title is empty".to_string(),
      WindowError::InvalidBitmap => "Invalid bitmap".to_string(),
      WindowError::UnsupportedPlatform => {
        "This operation is not supported on the current platform".to_string()
      }
    };
    Error::from_reason(message)
  }
}

pub trait NativeWindow {
  fn box_clone(&self) -> Box<dyn NativeWindow + Send + Sync>;
  fn title(&self) -> Result<String, WindowError>;
  fn x(&self) -> Result<i32, WindowError>;
  fn y(&self) -> Result<i32, WindowError>;
  fn width(&self) -> Result<u32, WindowError>;
  fn height(&self) -> Result<u32, WindowError>;
  fn is_focused(&self) -> Result<bool, WindowError>;
  fn capture_image(&self) -> Result<image::RgbaImage, WindowError>;
}

pub trait NativeWindowFactory {
  fn all_windows() -> Result<Vec<Window>, WindowError>
  where
    Self: Sized;
}

#[napi]
pub struct Window {
  native_window: Box<dyn NativeWindow + Send + Sync>,
}

impl Clone for Window {
  fn clone(&self) -> Self {
    Self {
      native_window: self.native_window.box_clone(),
    }
  }
}

#[napi]
impl Window {
  #[napi(constructor)]
  pub fn new() -> Self {
    panic!("Do not call `new Window()`. Use `Window.all()` to get window instances.");
  }

  #[napi]
  pub fn all() -> Result<Vec<Window>, Error> {
    #[cfg(target_os = "windows")]
    {
      WindowsWindow::all_windows().map_err(|e| e.into())
    }
    #[cfg(target_os = "linux")]
    {
      X11Window::all_windows().map_err(|e| e.into())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
      unsupported_backend::UnsupportedOSWindow::all_windows().map_err(|e| e.into())
    }
  }

  #[napi]
  pub fn title(&self) -> Result<String, Error> {
    Ok(self.native_window.title()?)
  }

  #[napi]
  pub fn x(&self) -> Result<i32, Error> {
    Ok(self.native_window.x()?)
  }

  #[napi]
  pub fn y(&self) -> Result<i32, Error> {
    Ok(self.native_window.y()?)
  }

  #[napi]
  pub fn width(&self) -> Result<u32, Error> {
    Ok(self.native_window.width()?)
  }

  #[napi]
  pub fn height(&self) -> Result<u32, Error> {
    Ok(self.native_window.height()?)
  }

  #[napi]
  pub fn is_focused(&self) -> Result<bool, Error> {
    Ok(self.native_window.is_focused()?)
  }

  #[napi(ts_return_type = "Promise<Image>")]
  pub fn capture_image(&self) -> AsyncTask<AsyncCaptureImage> {
    AsyncTask::new(AsyncCaptureImage::new(self.clone()))
  }
}

pub struct AsyncCaptureImage {
  window: Window,
}

impl AsyncCaptureImage {
  pub fn new(window: Window) -> Self {
    Self { window }
  }
}

#[napi]
impl Task for AsyncCaptureImage {
  type Output = Image;
  type JsValue = Image;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let rgba_image = self.window.native_window.capture_image()?;
    Ok(Image::from(rgba_image))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue, Error> {
    Ok(output)
  }
}
