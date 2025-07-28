#![cfg(target_os = "linux")]

use crate::native_api::x11_backend::{
  send_command_to_api_thread, WindowHandle, X11ApiCaptureWindowImageError, X11ApiCommand,
  X11ApiEnumerateWindowsError, X11ApiError, X11ApiGetWindowRectError, X11ApiGetWindowTitleError,
  X11ApiIsWindowFocusedError, X11ApiResponse, X11SendCommandToApiThreadError,
};
use crate::window::{NativeWindow, NativeWindowFactory, Window, WindowError};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum X11NativeWindowTitleError {
  ApiError(X11SendCommandToApiThreadError),
  GetWindowTitleError(X11ApiGetWindowTitleError),
  UnexpectedResponse,
}

impl Into<WindowError> for X11NativeWindowTitleError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum X11NativeWindowXError {
  ApiError(X11SendCommandToApiThreadError),
  GetWindowRectError(X11ApiGetWindowRectError),
  UnexpectedResponse,
}

impl Into<WindowError> for X11NativeWindowXError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum X11NativeWindowYError {
  ApiError(X11SendCommandToApiThreadError),
  GetWindowRectError(X11ApiGetWindowRectError),
  UnexpectedResponse,
}

impl Into<WindowError> for X11NativeWindowYError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum X11NativeWindowWidthError {
  ApiError(X11SendCommandToApiThreadError),
  GetWindowRectError(X11ApiGetWindowRectError),
  UnexpectedResponse,
}

impl Into<WindowError> for X11NativeWindowWidthError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum X11NativeWindowHeightError {
  ApiError(X11SendCommandToApiThreadError),
  GetWindowRectError(X11ApiGetWindowRectError),
  UnexpectedResponse,
}

impl Into<WindowError> for X11NativeWindowHeightError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum X11NativeWindowIsFocusedError {
  ApiError(X11SendCommandToApiThreadError),
  IsWindowFocusedError(X11ApiIsWindowFocusedError),
  UnexpectedResponse,
}

impl Into<WindowError> for X11NativeWindowIsFocusedError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum X11NativeWindowCaptureImageError {
  ApiError(X11SendCommandToApiThreadError),
  CaptureWindowImageError(X11ApiCaptureWindowImageError),
  UnexpectedResponse,
}

impl Into<WindowError> for X11NativeWindowCaptureImageError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum X11NativeWindowAllWindowsError {
  ApiError(X11SendCommandToApiThreadError),
  EnumerateWindowsError(X11ApiEnumerateWindowsError),
  UnexpectedResponse,
}

impl Into<WindowError> for X11NativeWindowAllWindowsError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

pub struct X11Window {
  handle: WindowHandle,
}

impl Clone for X11Window {
  fn clone(&self) -> Self {
    Self {
      handle: self.handle,
    }
  }
}

impl Into<Window> for X11Window {
  fn into(self) -> Window {
    Window {
      native_window: Box::new(self),
    }
  }
}

impl NativeWindow for X11Window {
  fn box_clone(&self) -> Box<dyn NativeWindow + Send + Sync> {
    Box::new(self.clone())
  }

  fn title(&self) -> Result<String, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::GetWindowTitle(self.handle)).map_err(|e| X11NativeWindowTitleError::ApiError(e).into())? {
      X11ApiResponse::WindowTitle(title) => Ok(title),
      X11ApiResponse::Error(X11ApiError::GetWindowTitle(e)) => {
        Err(X11NativeWindowTitleError::GetWindowTitleError(e).into())
      }
      _ => Err(X11NativeWindowTitleError::UnexpectedResponse.into()),
    }
  }

  fn x(&self) -> Result<i32, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::GetWindowRect(self.handle)).map_err(|e| X11NativeWindowXError::ApiError(e).into())? {
      X11ApiResponse::WindowRect(rect) => Ok(rect.left),
      X11ApiResponse::Error(X11ApiError::GetWindowRect(e)) => Err(X11NativeWindowXError::GetWindowRectError(e).into()),
      _ => Err(X11NativeWindowXError::UnexpectedResponse.into()),
    }
  }

  fn y(&self) -> Result<i32, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::GetWindowRect(self.handle)).map_err(|e| X11NativeWindowYError::ApiError(e).into())? {
      X11ApiResponse::WindowRect(rect) => Ok(rect.top),
      X11ApiResponse::Error(X11ApiError::GetWindowRect(e)) => Err(X11NativeWindowYError::GetWindowRectError(e).into()),
      _ => Err(X11NativeWindowYError::UnexpectedResponse.into()),
    }
  }

  fn width(&self) -> Result<u32, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::GetWindowRect(self.handle)).map_err(|e| X11NativeWindowWidthError::ApiError(e).into())? {
      X11ApiResponse::WindowRect(rect) => Ok((rect.right - rect.left) as u32),
      X11ApiResponse::Error(X11ApiError::GetWindowRect(e)) => Err(X11NativeWindowWidthError::GetWindowRectError(e).into()),
      _ => Err(X11NativeWindowWidthError::UnexpectedResponse.into()),
    }
  }

  fn height(&self) -> Result<u32, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::GetWindowRect(self.handle)).map_err(|e| X11NativeWindowHeightError::ApiError(e).into())? {
      X11ApiResponse::WindowRect(rect) => Ok((rect.bottom - rect.top) as u32),
      X11ApiResponse::Error(X11ApiError::GetWindowRect(e)) => Err(X11NativeWindowHeightError::GetWindowRectError(e).into()),
      _ => Err(X11NativeWindowHeightError::UnexpectedResponse.into()),
    }
  }

  fn is_focused(&self) -> Result<bool, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::IsWindowFocused(self.handle)).map_err(|e| X11NativeWindowIsFocusedError::ApiError(e).into())? {
      X11ApiResponse::WindowFocused(focused) => Ok(focused),
      X11ApiResponse::Error(X11ApiError::IsWindowFocused(e)) => Err(X11NativeWindowIsFocusedError::IsWindowFocusedError(e).into()),
      _ => Err(X11NativeWindowIsFocusedError::UnexpectedResponse.into()),
    }
  }

  fn capture_image(&self) -> Result<image::RgbaImage, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::CaptureWindowImage(self.handle)).map_err(|e| X11NativeWindowCaptureImageError::ApiError(e).into())? {
      X11ApiResponse::WindowImage(img) => Ok(img),
      X11ApiResponse::Error(X11ApiError::CaptureWindowImage(e)) => Err(X11NativeWindowCaptureImageError::CaptureWindowImageError(e).into()),
      _ => Err(X11NativeWindowCaptureImageError::UnexpectedResponse.into()),
    }
  }
}

impl NativeWindowFactory for X11Window {
  fn all_windows() -> Result<Vec<Window>, WindowError>
  where
    Self: Sized,
  {
    let response = send_command_to_api_thread(X11ApiCommand::EnumerateWindows).map_err(|e| X11NativeWindowAllWindowsError::ApiError(e).into())?;
    match response {
      X11ApiResponse::WindowList(hwnds_raw) => Ok(
        hwnds_raw
          .into_iter()
          .map(|handle| X11Window { handle }.into())
          .collect(),
      ),
      X11ApiResponse::Error(X11ApiError::EnumerateWindows(e)) => Err(X11NativeWindowAllWindowsError::EnumerateWindowsError(e).into()),
      _ => Err(X11NativeWindowAllWindowsError::UnexpectedResponse.into()),
    }
  }
}
