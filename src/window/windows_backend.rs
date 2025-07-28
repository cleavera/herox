#![cfg(target_os = "windows")]

use crate::native_api::windows_backend::{
  send_command_to_api_thread, WindowHandle, WindowsApiCommand, WindowsApiError, WindowsApiResponse, WindowsSendCommandToApiThreadError,
};
use crate::window::{NativeWindow, NativeWindowFactory, Window, WindowError};

pub struct WindowsWindow {
  handle: WindowHandle,
}

impl Clone for WindowsWindow {
  fn clone(&self) -> Self {
    Self {
      handle: self.handle,
    }
  }
}

impl Into<Window> for WindowsWindow {
  fn into(self) -> Window {
    Window {
      native_window: Box::new(self),
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub enum WindowsNativeWindowTitleError {
  UnexpectedResponse,
  ApiError(SendCommandToApiThreadError),
}

impl Into<WindowError> for WindowsNativeWindowTitleError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Copy, Debug)]
pub enum WindowsNativeWindowXError {
  GetRectError(WindowsApiGetWindowRectError),
  UnexpectedResponse,
  ApiError(SendCommandToApiThreadError),
}

impl Into<WindowError> for WindowsNativeWindowXError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Copy, Debug)]
pub enum WindowsNativeWindowYError {
  GetRectError(WindowsApiGetWindowRectError),
  UnexpectedResponse,
  ApiError(SendCommandToApiThreadError),
}

impl Into<WindowError> for WindowsNativeWindowYError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Copy, Debug)]
pub enum WindowsNativeWindowWidthError {
  GetRectError(WindowsApiGetWindowRectError),
  UnexpectedResponse,
  ApiError(SendCommandToApiThreadError),
}

impl Into<WindowError> for WindowsNativeWindowWidthError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Copy, Debug)]
pub enum WindowsNativeWindowHeightError {
  GetRectError(WindowsApiGetWindowRectError),
  UnexpectedResponse,
  ApiError(SendCommandToApiThreadError),
}

impl Into<WindowError> for WindowsNativeWindowHeightError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Copy, Debug)]
pub enum WindowsNativeWindowIsFocusedError {
  UnexpectedResponse,
  ApiError(SendCommandToApiThreadError),
}

impl Into<WindowError> for WindowsNativeWindowIsFocusedError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

#[derive(Clone, Copy, Debug)]
pub enum WindowsNativeWindowCaptureImageError {
  CaptureImageError(WindowsApiCaptureImageError),
  UnexpectedResponse,
  ApiError(SendCommandToApiThreadError),
}

impl Into<WindowError> for WindowsNativeWindowCaptureImageError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

impl NativeWindow for WindowsWindow {
  fn box_clone(&self) -> Box<dyn NativeWindow + Send + Sync> {
    Box::new(self.clone())
  }

  fn title(&self) -> Result<String, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::GetWindowTitle(self.handle)).map_err(|e| WindowsNativeWindowTitleError::ApiError(e).into())? {
      WindowsApiResponse::WindowTitle(title) => Ok(title),
      _ => Err(WindowsNativeWindowTitleError::UnexpectedResponse.into()),
    }
  }

  fn x(&self) -> Result<i32, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle)).map_err(|e| WindowsNativeWindowXError::ApiError(e).into())? {
      WindowsApiResponse::WindowRect(rect) => Ok(rect.left),
      WindowsApiResponse::Error(WindowsApiError::GetWindowRect(e)) => Err(WindowsNativeWindowXError::GetRectError(e).into()),
      _ => Err(WindowsNativeWindowXError::UnexpectedResponse.into()),
    }
  }

  fn y(&self) -> Result<i32, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle)).map_err(|e| WindowsNativeWindowYError::ApiError(e).into())? {
      WindowsApiResponse::WindowRect(rect) => Ok(rect.top),
      WindowsApiResponse::Error(WindowsApiError::GetWindowRect(e)) => Err(WindowsNativeWindowYError::GetRectError(e).into()),
      _ => Err(WindowsNativeWindowYError::UnexpectedResponse.into()),
    }
  }

  fn width(&self) -> Result<u32, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle)).map_err(|e| WindowsNativeWindowWidthError::ApiError(e).into())? {
      WindowsApiResponse::WindowRect(rect) => Ok((rect.right - rect.left) as u32),
      WindowsApiResponse::Error(WindowsApiError::GetWindowRect(e)) => Err(WindowsNativeWindowWidthError::GetRectError(e).into()),
      _ => Err(WindowsNativeWindowWidthError::UnexpectedResponse.into()),
    }
  }

  fn height(&self) -> Result<u32, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle)).map_err(|e| WindowsNativeWindowHeightError::ApiError(e).into())? {
      WindowsApiResponse::WindowRect(rect) => Ok((rect.bottom - rect.top) as u32),
      WindowsApiResponse::Error(WindowsApiError::GetWindowRect(e)) => Err(WindowsNativeWindowHeightError::GetRectError(e).into()),
      _ => Err(WindowsNativeWindowHeightError::UnexpectedResponse.into()),
    }
  }

  fn is_focused(&self) -> Result<bool, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::IsWindowFocused(self.handle)).map_err(|e| WindowsNativeWindowIsFocusedError::ApiError(e).into())? {
      WindowsApiResponse::WindowFocused(focused) => Ok(focused),
      _ => Err(WindowsNativeWindowIsFocusedError::UnexpectedResponse.into()),
    }
  }

  fn capture_image(&self) -> Result<image::RgbaImage, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::CaptureWindowImage(self.handle)).map_err(|e| WindowsNativeWindowCaptureImageError::ApiError(e).into())? {
      WindowsApiResponse::WindowImage(img) => Ok(img),
      WindowsApiResponse::Error(WindowsApiError::CaptureWindowImage(e)) => Err(WindowsNativeWindowCaptureImageError::CaptureImageError(e).into()),
      _ => Err(WindowsNativeWindowCaptureImageError::UnexpectedResponse.into()),
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub enum WindowsNativeWindowAllWindowsError {
  EnumerateWindowsError(WindowsApiEnumerateWindowsError),
  UnexpectedResponse,
  ApiError(SendCommandToApiThreadError),
}

impl Into<WindowError> for WindowsNativeWindowAllWindowsError {
  fn into(self) -> WindowError {
    WindowError::from_reason(format!("{:?}", self))
  }
}

impl NativeWindowFactory for WindowsWindow {
  fn all_windows() -> Result<Vec<Window>, WindowError>
  where
    Self: Sized,
  {
    let response = send_command_to_api_thread(WindowsApiCommand::EnumerateWindows).map_err(|e| WindowsNativeWindowAllWindowsError::ApiError(e).into())?;
    match response {
      WindowsApiResponse::WindowList(hwnds_raw) => Ok(
        hwnds_raw
          .into_iter()
          .map(|handle| WindowsWindow { handle }.into())
          .collect(),
      ),
      WindowsApiResponse::Error(WindowsApiError::EnumerateWindows(e)) => Err(WindowsNativeWindowAllWindowsError::EnumerateWindowsError(e).into()),
      _ => Err(WindowsNativeWindowAllWindowsError::UnexpectedResponse.into()),
    }
  }
}
