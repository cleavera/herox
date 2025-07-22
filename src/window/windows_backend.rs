#![cfg(target_os = "windows")]

use crate::native_api::windows_backend::{
  send_command_to_api_thread, WindowHandle, WindowsApiCommand, WindowsApiResponse,
};
use crate::window::{NativeWindow, Window, WindowError};

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

impl NativeWindow for WindowsWindow {
  fn box_clone(&self) -> Box<dyn NativeWindow + Send + Sync> {
    Box::new(self.clone())
  }

  fn title(&self) -> Result<String, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::GetWindowTitle(self.handle))? {
      WindowsApiResponse::WindowTitle(title) => Ok(title),
      WindowsApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for GetWindowTitle".to_string(),
      )),
    }
  }

  fn x(&self) -> Result<i32, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle))? {
      WindowsApiResponse::WindowRect(rect) => Ok(rect.left),
      WindowsApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for GetWindowRect".to_string(),
      )),
    }
  }

  fn y(&self) -> Result<i32, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle))? {
      WindowsApiResponse::WindowRect(rect) => Ok(rect.top),
      WindowsApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for GetWindowRect".to_string(),
      )),
    }
  }

  fn width(&self) -> Result<u32, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle))? {
      WindowsApiResponse::WindowRect(rect) => Ok((rect.right - rect.left) as u32),
      WindowsApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for GetWindowRect".to_string(),
      )),
    }
  }

  fn height(&self) -> Result<u32, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle))? {
      WindowsApiResponse::WindowRect(rect) => Ok((rect.bottom - rect.top) as u32),
      WindowsApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for GetWindowRect".to_string(),
      )),
    }
  }

  fn is_focused(&self) -> Result<bool, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::IsWindowFocused(self.handle))? {
      WindowsApiResponse::WindowFocused(focused) => Ok(focused),
      WindowsApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for IsWindowFocused".to_string(),
      )),
    }
  }

  fn capture_image(&self) -> Result<image::RgbaImage, WindowError> {
    match send_command_to_api_thread(WindowsApiCommand::CaptureWindowImage(self.handle))? {
      WindowsApiResponse::WindowImage(img) => Ok(img),
      WindowsApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for CaptureWindowImage".to_string(),
      )),
    }
  }
}
