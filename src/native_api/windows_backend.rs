#![cfg(target_os = "windows")]

use windows::{
  Win32::Foundation::{GetLastError, BOOL, HWND, LPARAM, RECT, TRUE},
  Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
    GetWindowDC, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, SRCCOPY,
  },
  Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowRect, GetWindowTextW, IsIconic, IsWindow,
    IsWindowVisible,
  },
};

use crate::window::WindowError;
use core::ffi::c_void;
use once_cell::sync::OnceCell;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Once;
use std::thread;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WindowHandle(u64);

impl WindowHandle {
  pub fn new(hwnd: HWND) -> Self {
    Self(hwnd.0 as u64)
  }

  pub fn as_hwnd(&self) -> HWND {
    HWND(self.0 as *mut c_void)
  }
}

pub enum WindowsApiCommand {
  EnumerateWindows,
  GetWindowTitle(WindowHandle),
  GetWindowRect(WindowHandle),
  IsWindowFocused(WindowHandle),
  CaptureWindowImage(WindowHandle),
  Shutdown,
}

pub enum WindowsApiResponse {
  WindowList(Vec<WindowHandle>),
  WindowTitle(String),
  WindowRect(RECT),
  WindowFocused(bool),
  WindowImage(image::RgbaImage),
  Error(WindowError),
  Acknowledgement,
}

fn windows_api_thread_main(receiver: Receiver<(WindowsApiCommand, Sender<WindowsApiResponse>)>) {
  while let Ok((command, response_sender)) = receiver.recv() {
    match command {
      WindowsApiCommand::EnumerateWindows => {
        let mut hwnds: Vec<WindowHandle> = Vec::new();
        let result = unsafe {
          EnumWindows(
            Some(enum_windows_proc_for_thread),
            LPARAM(&mut hwnds as *mut _ as isize),
          )
        };
        if result.is_err() {
          response_sender
            .send(WindowsApiResponse::Error(WindowError::ApiError(
              "EnumWindows failed".to_string(),
            )))
            .ok();
        } else {
          response_sender
            .send(WindowsApiResponse::WindowList(hwnds))
            .ok();
        }
      }
      WindowsApiCommand::GetWindowTitle(handle) => {
        let hwnd = handle.as_hwnd();
        let mut text: [u16; 512] = [0; 512];
        let len = unsafe { GetWindowTextW(hwnd, &mut text) };
        if len == 0 {
          response_sender
            .send(WindowsApiResponse::WindowTitle("".to_string()))
            .ok();
        } else {
          response_sender
            .send(WindowsApiResponse::WindowTitle(String::from_utf16_lossy(
              &text[..len as usize],
            )))
            .ok();
        }
      }
      WindowsApiCommand::GetWindowRect(handle) => {
        let hwnd = handle.as_hwnd();
        let mut rect = RECT::default();
        if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
          response_sender
            .send(WindowsApiResponse::Error(WindowError::ApiError(
              "Failed to get window rect".to_string(),
            )))
            .ok();
        } else {
          response_sender
            .send(WindowsApiResponse::WindowRect(rect))
            .ok();
        }
      }
      WindowsApiCommand::IsWindowFocused(handle) => {
        let hwnd = handle.as_hwnd();
        let foreground_window = unsafe { GetForegroundWindow() };
        response_sender
          .send(WindowsApiResponse::WindowFocused(hwnd == foreground_window))
          .ok();
      }
      WindowsApiCommand::CaptureWindowImage(handle) => {
        let hwnd = handle.as_hwnd();
        match capture_window_image_internal(hwnd) {
          Ok(img) => {
            response_sender
              .send(WindowsApiResponse::WindowImage(img))
              .ok();
          }
          Err(e) => {
            response_sender.send(WindowsApiResponse::Error(e)).ok();
          }
        }
      }
      WindowsApiCommand::Shutdown => {
        response_sender
          .send(WindowsApiResponse::Acknowledgement)
          .ok();
        break;
      }
    }
  }
}

fn capture_window_image_internal(hwnd: HWND) -> Result<image::RgbaImage, WindowError> {
  let mut rect = RECT::default();
  if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
    return Err(WindowError::ApiError(
      "Failed to get window rect".to_string(),
    ));
  }

  let width = (rect.right - rect.left) as i32;
  let height = (rect.bottom - rect.top) as i32;

  if unsafe { IsIconic(hwnd) }.as_bool() {
    return Err(WindowError::ApiError(
      "Window is minimized, cannot capture image".to_string(),
    ));
  }

  if unsafe { IsWindow(hwnd) }.as_bool() {
    return Err(WindowError::ApiError(
      "Window handle is no longer valid".to_string(),
    ));
  }

  let hdc = unsafe { GetWindowDC(hwnd) };
  if hdc.0.is_null() {
    return Err(WindowError::ApiError(
      "Failed to get device context".to_string(),
    ));
  }

  let mem_dc = unsafe { CreateCompatibleDC(hdc) };
  if mem_dc.0.is_null() {
    unsafe {
      let _ = ReleaseDC(hwnd, hdc);
    };
    let error_code = unsafe { GetLastError() };
    return Err(WindowError::ApiError(format!(
      "CreateCompatibleDC failed with error code {}, this may be a GDI resource leak",
      error_code.0
    )));
  }

  let mem_bitmap = unsafe { CreateCompatibleBitmap(hdc, width, height) };
  if mem_bitmap.0.is_null() {
    unsafe {
      let _ = DeleteDC(mem_dc);
    };
    unsafe {
      let _ = ReleaseDC(hwnd, hdc);
    };
    return Err(WindowError::ApiError(
      "Failed to create compatible bitmap".to_string(),
    ));
  }

  let old_bitmap = unsafe { SelectObject(mem_dc, mem_bitmap) };

  if unsafe { BitBlt(mem_dc, 0, 0, width, height, hdc, 0, 0, SRCCOPY) }.is_err() {
    unsafe {
      let _ = SelectObject(mem_dc, old_bitmap);
    };
    unsafe {
      let _ = DeleteObject(mem_bitmap);
    };
    unsafe {
      let _ = DeleteDC(mem_dc);
    };
    unsafe {
      let _ = ReleaseDC(hwnd, hdc);
    };
    return Err(WindowError::ApiError(
      "Failed to copy screen to bitmap".to_string(),
    ));
  }

  let mut bmi = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: width,
      biHeight: -height,
      biPlanes: 1,
      biBitCount: 32,
      biCompression: 0,
      biSizeImage: 0,
      biXPelsPerMeter: 0,
      biYPelsPerMeter: 0,
      biClrUsed: 0,
      biClrImportant: 0,
    },
    bmiColors: [Default::default(); 1],
  };

  let mut buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

  let result = unsafe {
    GetDIBits(
      hdc,
      mem_bitmap,
      0,
      height as u32,
      Some(buffer.as_mut_ptr() as *mut _),
      &mut bmi as *mut _,
      DIB_RGB_COLORS,
    )
  };

  if result == 0 {
    return Err(WindowError::ApiError("Failed to get DIBits".to_string()));
  }

  for chunk in buffer.chunks_mut(4) {
    chunk.swap(0, 2);
  }

  image::RgbaImage::from_raw(width as u32, height as u32, buffer)
    .ok_or_else(|| WindowError::InvalidBitmap)
}

static WINDOWS_API_SENDER: OnceCell<Sender<(WindowsApiCommand, Sender<WindowsApiResponse>)>> =
  OnceCell::new();
static INIT_WINDOWS_API_THREAD: Once = Once::new();

pub fn send_command_to_api_thread(
  command: WindowsApiCommand,
) -> Result<WindowsApiResponse, WindowError> {
  INIT_WINDOWS_API_THREAD.call_once(|| {
    let (sender, receiver) = channel();
    WINDOWS_API_SENDER.set(sender).unwrap();
    thread::spawn(move || windows_api_thread_main(receiver));
  });

  let (response_sender, response_receiver) = channel();
  let sender = WINDOWS_API_SENDER.get().unwrap();
  sender
    .send((command, response_sender))
    .map_err(|e| WindowError::ApiError(format!("Failed to send command to API thread: {}", e)))?;
  Ok(response_receiver.recv().map_err(|e| {
    WindowError::ApiError(format!("Failed to receive response from API thread: {}", e))
  })?)
}

pub extern "system" fn enum_windows_proc_for_thread(hwnd: HWND, lparam: LPARAM) -> BOOL {
  let hwnds = unsafe { &mut *(lparam.0 as *mut Vec<WindowHandle>) };
  if unsafe { IsWindowVisible(hwnd) } == TRUE {
    let mut text: [u16; 512] = [0; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut text) };
    if len > 0 {
      hwnds.push(WindowHandle::new(hwnd));
    }
  }
  TRUE
}
