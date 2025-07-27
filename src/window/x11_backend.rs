#![cfg(target_os = "linux")]

use crate::native_api::x11_backend::{
  send_command_to_api_thread, WindowHandle, X11ApiCommand, X11ApiResponse,
};
use crate::window::{NativeWindow, NativeWindowFactory, Window, WindowError};

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
    match send_command_to_api_thread(X11ApiCommand::GetWindowTitle(self.handle))? {
      X11ApiResponse::WindowTitle(title) => Ok(title),
      X11ApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for GetWindowTitle".to_string(),
      )),
    }
  }

  fn x(&self) -> Result<i32, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::GetWindowRect(self.handle))? {
      X11ApiResponse::WindowRect(rect) => Ok(rect.left),
      X11ApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for GetWindowRect".to_string(),
      )),
    }
  }

  fn y(&self) -> Result<i32, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::GetWindowRect(self.handle))? {
      X11ApiResponse::WindowRect(rect) => Ok(rect.top),
      X11ApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for GetWindowRect".to_string(),
      )),
    }
  }

  fn width(&self) -> Result<u32, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::GetWindowRect(self.handle))? {
      X11ApiResponse::WindowRect(rect) => Ok((rect.right - rect.left) as u32),
      X11ApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for GetWindowRect".to_string(),
      )),
    }
  }

  fn height(&self) -> Result<u32, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::GetWindowRect(self.handle))? {
      X11ApiResponse::WindowRect(rect) => Ok((rect.bottom - rect.top) as u32),
      X11ApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for GetWindowRect".to_string(),
      )),
    }
  }

  fn is_focused(&self) -> Result<bool, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::IsWindowFocused(self.handle))? {
      X11ApiResponse::WindowFocused(focused) => Ok(focused),
      X11ApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for IsWindowFocused".to_string(),
      )),
    }
  }

  fn capture_image(&self) -> Result<image::RgbaImage, WindowError> {
    match send_command_to_api_thread(X11ApiCommand::CaptureWindowImage(self.handle))? {
      X11ApiResponse::WindowImage(img) => Ok(img),
      X11ApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for CaptureWindowImage".to_string(),
      )),
    }
  }
}

impl NativeWindowFactory for X11Window {
  fn all_windows() -> Result<Vec<Window>, WindowError>
  where
    Self: Sized,
  {
    let response = send_command_to_api_thread(X11ApiCommand::EnumerateWindows)?;
    match response {
      X11ApiResponse::WindowList(hwnds_raw) => Ok(
        hwnds_raw
          .into_iter()
          .map(|handle| X11Window { handle }.into())
          .collect(),
      ),
      X11ApiResponse::Error(e) => Err(e),
      _ => Err(WindowError::ApiError(
        "Unexpected response for EnumerateWindows".to_string(),
      )),
    }
  }
}
