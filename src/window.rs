use napi::{bindgen_prelude::AsyncTask, Env, Error, Task};

use crate::image::Image;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::WindowsWindow;

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
        WindowError::UnsupportedPlatform => "This operation is not supported on the current platform".to_string(),
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
        #[cfg(target_os = "windows")]
        {
            panic!("Window::new() is not supported directly. Use Window::all() or specific platform constructors.");
        }
        #[cfg(not(target_os = "windows"))]
        {
            Window {
                native_window: Box::new(UnsupportedOSWindow),
            }
        }
    }

    // Internal constructor for use by `Window::all()`
    #[cfg(target_os = "windows")]
    pub(crate) fn from_native_impl(native_window: WindowsWindow) -> Self {
        Window {
            native_window: Box::new(native_window),
        }
    }

    #[napi]
    pub fn all() -> Result<Vec<Window>, Error> {
        #[cfg(target_os = "windows")]
        {
            let mut windows: Vec<Window> = Vec::new();
            unsafe {
                windows::EnumWindows(Some(windows::enum_windows_proc), windows::LPARAM(&mut windows as *mut _ as isize));
            }
            Ok(windows)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(WindowError::UnsupportedPlatform.into())
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
