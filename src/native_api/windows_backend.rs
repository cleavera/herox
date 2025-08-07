#![cfg(target_os = "windows")]

use windows::Win32::{
  Foundation::{GetLastError, BOOL, HWND, LPARAM, RECT, TRUE},
  Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetWindowDC, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ, SRCCOPY
  },
  UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowRect, GetWindowTextW, IsIconic, IsWindow,
    IsWindowVisible,
  },
};

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

#[derive(Copy, Clone, Debug)]
pub enum WindowsApiEnumerateWindowsError {
  Generic(u32),
}

#[derive(Copy, Clone, Debug)]
pub enum WindowsApiGetWindowRectError {
  Generic(u32),
}

#[derive(Copy, Clone, Debug)]
pub enum WindowsApiCaptureWindowImageError {
  Generic(u32),
  GetWindowRectError(WindowsApiGetWindowRectError),
  IsMinimized,
  HandleNoLongerValid,
  GetWindowDcError(u32),
  CreateCompatibleDcError(u32),
  CreateCompatibleBitmapError(u32),
  CopyBitmapError(u32),
  DiBitsToBufferError(u32),
  InvalidBitmap,
}

#[derive(Copy, Clone, Debug)]
pub enum WindowsApiError {
  EnumerateWindows(WindowsApiEnumerateWindowsError),
  GetWindowRect(WindowsApiGetWindowRectError),
  CaptureWindowImage(WindowsApiCaptureWindowImageError),
}

pub enum WindowsApiResponse {
  WindowList(Vec<WindowHandle>),
  WindowTitle(String),
  WindowRect(RECT),
  WindowFocused(bool),
  WindowImage(image::RgbaImage),
  Error(WindowsApiError),
  Acknowledgement,
}

fn get_error_code() -> u32 {
  let error_code = unsafe { GetLastError() };

  error_code.0
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
            .send(WindowsApiResponse::Error(
              WindowsApiError::EnumerateWindows(WindowsApiEnumerateWindowsError::Generic(
                get_error_code(),
              )),
            ))
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
        match get_window_rect(hwnd) {
          Ok(rect) => response_sender
            .send(WindowsApiResponse::WindowRect(rect))
            .ok(),
          Err(e) => response_sender
            .send(WindowsApiResponse::Error(WindowsApiError::GetWindowRect(e)))
            .ok(),
        };
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
            response_sender
              .send(WindowsApiResponse::Error(
                WindowsApiError::CaptureWindowImage(e),
              ))
              .ok();
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

fn get_window_rect(hwnd: HWND) -> Result<RECT, WindowsApiGetWindowRectError> {
  let mut rect = RECT::default();
  if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
    return Err(WindowsApiGetWindowRectError::Generic(get_error_code()));
  } else {
    return Ok(rect);
  }
}

pub struct WindowDeviceContext {
  pub hwnd: HWND,
  pub hdc: HDC,
}

impl WindowDeviceContext {
  pub fn new(hwnd: HWND) -> Result<Self, u32> {
    let hdc = unsafe { GetWindowDC(hwnd) };
    let dc = Self { hwnd, hdc };

    if hdc.0.is_null() {
      return Err(get_error_code());
    }

    Ok(dc)
  }

  pub fn create_compatible_dc(&self) -> Result<CompatibleDeviceContext, u32> {
    CompatibleDeviceContext::new(self.hdc)
  }

  pub fn create_bitmap(&self, width: i32, height: i32) -> Result<CompatibleBitmap, u32> {
    CompatibleBitmap::new(self.hdc, width, height)
  }
}

impl Drop for WindowDeviceContext {
  fn drop(&mut self) {
    unsafe {
      ReleaseDC(self.hwnd, self.hdc);
    }
  }
}

pub struct CompatibleDeviceContext {
  pub hdc: HDC,
}

impl CompatibleDeviceContext {
  pub fn new(hdc: HDC) -> Result<Self, u32> {
    let mem_dc = unsafe { CreateCompatibleDC(hdc) };
    let dc = Self { hdc: mem_dc };

    if mem_dc.0.is_null() {
      return Err(get_error_code());
    }

    Ok(dc)
  }
}

impl Drop for CompatibleDeviceContext {
  fn drop(&mut self) {
    unsafe {
      let _ = DeleteDC(self.hdc);
    }
  }
}

pub struct CompatibleBitmap {
  pub bitmap: HBITMAP,
}

impl CompatibleBitmap {
  pub fn new(hdc: HDC, width: i32, height: i32) -> Result<Self, u32> {
    let bitmap = unsafe { CreateCompatibleBitmap(hdc, width, height) };
    let bmp = Self { bitmap };

    if bitmap.0.is_null() {
      return Err(get_error_code());
    }

    Ok(bmp)
  }
}

impl Drop for CompatibleBitmap {
  fn drop(&mut self) {
    unsafe {
      let _ = DeleteObject(self.bitmap);
    }
  }
}



fn capture_window_image_internal(
  hwnd: HWND,
) -> Result<image::RgbaImage, WindowsApiCaptureWindowImageError> {
  let rect =
    get_window_rect(hwnd).map_err(|e| WindowsApiCaptureWindowImageError::GetWindowRectError(e))?;
  let width = (rect.right - rect.left) as i32;
  let height = (rect.bottom - rect.top) as i32;

  if unsafe { IsIconic(hwnd) }.as_bool() {
    return Err(WindowsApiCaptureWindowImageError::IsMinimized);
  }

  if !unsafe { IsWindow(hwnd) }.as_bool() {
    return Err(WindowsApiCaptureWindowImageError::HandleNoLongerValid);
  }

  let hdc = WindowDeviceContext::new(hwnd)
    .map_err(|e| WindowsApiCaptureWindowImageError::GetWindowDcError(e))?;
  let mem_dc = hdc
    .create_compatible_dc()
    .map_err(|e| WindowsApiCaptureWindowImageError::CreateCompatibleDcError(e))?;
  let mem_bitmap = hdc
    .create_bitmap(width, height)
    .map_err(|e| WindowsApiCaptureWindowImageError::CreateCompatibleBitmapError(e))?;

  let old_bitmap = unsafe { SelectObject(mem_dc.hdc, mem_bitmap.bitmap) };

  if unsafe { BitBlt(mem_dc.hdc, 0, 0, width, height, hdc.hdc, 0, 0, SRCCOPY) }.is_err() {
    let error_code = get_error_code();
    unsafe { SelectObject(mem_dc.hdc, old_bitmap) };
    return Err(WindowsApiCaptureWindowImageError::CopyBitmapError(
      error_code,
    ));
  }

  unsafe { SelectObject(mem_dc.hdc, old_bitmap) };

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
      mem_dc.hdc,
      mem_bitmap.bitmap,
      0,
      height as u32,
      Some(buffer.as_mut_ptr() as *mut _),
      &mut bmi as *mut _,
      DIB_RGB_COLORS,
    )
  };

  if result == 0 {
    return Err(WindowsApiCaptureWindowImageError::DiBitsToBufferError(
      get_error_code(),
    ));
  }

  for chunk in buffer.chunks_mut(4) {
    chunk.swap(0, 2);
  }

  image::RgbaImage::from_raw(width as u32, height as u32, buffer)
    .ok_or_else(|| WindowsApiCaptureWindowImageError::InvalidBitmap)
}

static WINDOWS_API_SENDER: OnceCell<Sender<(WindowsApiCommand, Sender<WindowsApiResponse>)>> =
  OnceCell::new();
static INIT_WINDOWS_API_THREAD: Once = Once::new();

#[derive(Clone, Copy, Debug)]
pub enum WindowsSendCommandToApiThreadError {
  Send,
  Receive,
}

pub fn send_command_to_api_thread(
  command: WindowsApiCommand,
) -> Result<WindowsApiResponse, WindowsSendCommandToApiThreadError> {
  INIT_WINDOWS_API_THREAD.call_once(|| {
    let (sender, receiver) = channel();
    WINDOWS_API_SENDER.set(sender).unwrap();
    thread::spawn(move || windows_api_thread_main(receiver));
  });

  let (response_sender, response_receiver) = channel();
  let sender = WINDOWS_API_SENDER.get().unwrap();
  sender
    .send((command, response_sender))
    .map_err(|_| WindowsSendCommandToApiThreadError::Send)?;
  Ok(
    response_receiver
      .recv()
      .map_err(|_| WindowsSendCommandToApiThreadError::Receive)?,
  )
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
