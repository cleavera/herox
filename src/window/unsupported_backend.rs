use crate::window::{NativeWindow, WindowError};

pub struct UnsupportedOSWindow;

impl NativeWindow for UnsupportedOSWindow {
  fn box_clone(&self) -> Box<dyn NativeWindow + Send + Sync> {
    Box::new(UnsupportedOSWindow)
  }

  fn title(&self) -> Result<String, WindowError> {
    Err(WindowError::UnsupportedPlatform)
  }

  fn x(&self) -> Result<i32, WindowError> {
    Err(WindowError::UnsupportedPlatform)
  }

  fn y(&self) -> Result<i32, WindowError> {
    Err(WindowError::UnsupportedPlatform)
  }

  fn width(&self) -> Result<u32, WindowError> {
    Err(WindowError::UnsupportedPlatform)
  }

  fn height(&self) -> Result<u32, WindowError> {
    Err(WindowError::UnsupportedPlatform)
  }

  fn is_focused(&self) -> Result<bool, WindowError> {
    Err(WindowError::UnsupportedPlatform)
  }

  fn capture_image(&self) -> Result<image::RgbaImage, WindowError> {
    Err(WindowError::UnsupportedPlatform)
  }
}
