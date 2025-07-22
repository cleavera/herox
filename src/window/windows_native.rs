use windows::{
    Win32::Foundation::{BOOL, HWND, LPARAM, RECT, TRUE, FALSE},
    Win32::Graphics::Gdi::{BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, SRCCOPY, GetDIBits, ReleaseDC, CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, BitBlt, DeleteDC, DeleteObject, GetWindowDC},
    Win32::UI::WindowsAndMessaging::{GetWindowTextW, GetWindowRect, IsWindowVisible, GetForegroundWindow},
};

use crate::window::{NativeWindow, WindowError};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WindowHandle(u64);

impl WindowHandle {
    fn new(hwnd: HWND) -> Self {
        Self(hwnd.0 as u64)
    }

    fn as_hwnd(&self) -> HWND {
        HWND(self.0 as isize)
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
                    windows::Win32::UI::WindowsAndMessaging::EnumWindows(
                        Some(enum_windows_proc_for_thread),
                        LPARAM(&mut hwnds as *mut _ as isize),
                    )
                };
                if result == FALSE {
                    response_sender.send(WindowsApiResponse::Error(WindowError::ApiError("EnumWindows failed".to_string()))).ok();
                } else {
                    response_sender.send(WindowsApiResponse::WindowList(hwnds)).ok();
                }
            },
            WindowsApiCommand::GetWindowTitle(handle) => {
                let hwnd = handle.as_hwnd();
                let mut text: [u16; 512] = [0; 512];
                let len = unsafe { GetWindowTextW(hwnd, &mut text) };
                if len == 0 {
                    response_sender.send(WindowsApiResponse::WindowTitle("".to_string())).ok();
                } else {
                    response_sender.send(WindowsApiResponse::WindowTitle(String::from_utf16_lossy(&text[..len as usize]))).ok();
                }
            },
            WindowsApiCommand::GetWindowRect(handle) => {
                let hwnd = handle.as_hwnd();
                let mut rect = RECT::default();
                if unsafe { GetWindowRect(hwnd, &mut rect) } == FALSE {
                    response_sender.send(WindowsApiResponse::Error(WindowError::ApiError("Failed to get window rect".to_string()))).ok();
                } else {
                    response_sender.send(WindowsApiResponse::WindowRect(rect)).ok();
                }
            },
            WindowsApiCommand::IsWindowFocused(handle) => {
                let hwnd = handle.as_hwnd();
                let foreground_window = unsafe { GetForegroundWindow() };
                response_sender.send(WindowsApiResponse::WindowFocused(hwnd == foreground_window)).ok();
            },
            WindowsApiCommand::CaptureWindowImage(handle) => {
                let hwnd = handle.as_hwnd();
                match capture_window_image_internal(hwnd) {
                    Ok(img) => response_sender.send(WindowsApiResponse::WindowImage(img)).ok(),
                    Err(e) => response_sender.send(WindowsApiResponse::Error(e)).ok(),
                }
            },
            WindowsApiCommand::Shutdown => {
                response_sender.send(WindowsApiResponse::Ack).ok();
                break; // Exit the thread loop
            },
        }
    }
}

fn capture_window_image_internal(hwnd: HWND) -> Result<image::RgbaImage, WindowError> {
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect) } == FALSE {
        return Err(WindowError::ApiError("Failed to get window rect".to_string()));
    }

    let width = (rect.right - rect.left) as i32;
    let height = (rect.bottom - rect.top) as i32;

    let hdc = unsafe { GetWindowDC(hwnd) };
    if hdc.0.is_null() {
        return Err(WindowError::ApiError("Failed to get device context".to_string()));
    }

    let mem_dc = unsafe { CreateCompatibleDC(hdc) };
    if mem_dc.0.is_null() {
        unsafe { ReleaseDC(hwnd, hdc) };
        return Err(WindowError::ApiError("Failed to create compatible DC".to_string()));
    }

    let mem_bitmap = unsafe { CreateCompatibleBitmap(hdc, width, height) };
    if mem_bitmap.0.is_null() {
        unsafe { DeleteDC(mem_dc) };
        unsafe { ReleaseDC(hwnd, hdc) };
        return Err(WindowError::ApiError("Failed to create compatible bitmap".to_string()));
    }

    let old_bitmap = unsafe { SelectObject(mem_dc, mem_bitmap) };

    if unsafe { BitBlt(mem_dc, 0, 0, width, height, hdc, 0, 0, SRCCOPY) } == FALSE {
        unsafe { SelectObject(mem_dc, old_bitmap) };
        unsafe { DeleteObject(mem_bitmap) };
        unsafe { DeleteDC(mem_dc) };
        unsafe { ReleaseDC(hwnd, hdc) };
        return Err(WindowError::ApiError("Failed to copy screen to bitmap".to_string()));
    }

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0, // BI_RGB
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [Default::default(); 1],
    };

    let mut buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

    let result = unsafe { GetDIBits(
        hdc,
        mem_bitmap,
        0,
        height as u32,
        Some(buffer.as_mut_ptr() as *mut _),
        &mut bmi as *mut _,
        DIB_RGB_COLORS,
    ) };

    if result == 0 {
        return Err(WindowError::ApiError("Failed to get DIBits".to_string()));
    }

    // BGRA to RGBA
    for chunk in buffer.chunks_mut(4) {
        chunk.swap(0, 2);
    }

    image::RgbaImage::from_raw(width as u32, height as u32, buffer)
        .ok_or_else(|| WindowError::InvalidBitmap)
}

static mut WINDOWS_API_SENDER: Option<Sender<(WindowsApiCommand, Sender<WindowsApiResponse>)>> = None;
static INIT_WINDOWS_API_THREAD: Once = Once::new();

pub fn send_command_to_api_thread(command: WindowsApiCommand) -> Result<WindowsApiResponse, WindowError> {
    INIT_WINDOWS_API_THREAD.call_once(|| {
        let (sender, receiver) = channel();
        unsafe {
            WINDOWS_API_SENDER = Some(sender);
        }
        thread::spawn(move || windows_api_thread_main(receiver));
    });

    let (response_sender, response_receiver) = channel();
    unsafe {
        if let Some(sender) = &WINDOWS_API_SENDER {
            sender.send((command, response_sender)).map_err(|e| WindowError::ApiError(format!("Failed to send command to API thread: {}", e)))?;
            response_receiver.recv().map_err(|e| WindowError::ApiError(format!("Failed to receive response from API thread: {}", e)))?
        } else {
            Err(WindowError::ApiError("Windows API thread not initialized after call_once".to_string()))
        }
    }
}

#[cfg(target_os = "windows")]
pub struct WindowsWindow {
    handle: WindowHandle,
}

#[cfg(target_os = "windows")]
impl Clone for WindowsWindow {
    fn clone(&self) -> Self {
        Self { handle: self.handle }
    }
}

#[cfg(target_os = "windows")]
impl NativeWindow for WindowsWindow {
    fn box_clone(&self) -> Box<dyn NativeWindow + Send + Sync> {
        Box::new(self.clone())
    }

    fn title(&self) -> Result<String, WindowError> {
        match send_command_to_api_thread(WindowsApiCommand::GetWindowTitle(self.handle))? {
            WindowsApiResponse::WindowTitle(title) => Ok(title),
            WindowsApiResponse::Error(e) => Err(e),
            _ => Err(WindowError::ApiError("Unexpected response for GetWindowTitle".to_string())),
        }
    }

    fn x(&self) -> Result<i32, WindowError> {
        match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle))? {
            WindowsApiResponse::WindowRect(rect) => Ok(rect.left),
            WindowsApiResponse::Error(e) => Err(e),
            _ => Err(WindowError::ApiError("Unexpected response for GetWindowRect".to_string())),
        }
    }

    fn y(&self) -> Result<i32, WindowError> {
        match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle))? {
            WindowsApiResponse::WindowRect(rect) => Ok(rect.top),
            WindowsApiResponse::Error(e) => Err(e),
            _ => Err(WindowError::ApiError("Unexpected response for GetWindowRect".to_string())),
        }
    }

    fn width(&self) -> Result<u32, WindowError> {
        match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle))? {
            WindowsApiResponse::WindowRect(rect) => Ok((rect.right - rect.left) as u32),
            WindowsApiResponse::Error(e) => Err(e),
            _ => Err(WindowError::ApiError("Unexpected response for GetWindowRect".to_string())),
        }
    }

    fn height(&self) -> Result<u32, WindowError> {
        match send_command_to_api_thread(WindowsApiCommand::GetWindowRect(self.handle))? {
            WindowsApiResponse::WindowRect(rect) => Ok((rect.bottom - rect.top) as u32),
            WindowsApiResponse::Error(e) => Err(e),
            _ => Err(WindowError::ApiError("Unexpected response for GetWindowRect".to_string())),
        }
    }

    fn is_focused(&self) -> Result<bool, WindowError> {
        match send_command_to_api_thread(WindowsApiCommand::IsWindowFocused(self.handle))? {
            WindowsApiResponse::WindowFocused(focused) => Ok(focused),
            WindowsApiResponse::Error(e) => Err(e),
            _ => Err(WindowError::ApiError("Unexpected response for IsWindowFocused".to_string())),
        }
    }

    fn capture_image(&self) -> Result<image::RgbaImage, WindowError> {
        match send_command_to_api_thread(WindowsApiCommand::CaptureWindowImage(self.handle))? {
            WindowsApiResponse::WindowImage(img) => Ok(img),
            WindowsApiResponse::Error(e) => Err(e),
            _ => Err(WindowError::ApiError("Unexpected response for CaptureWindowImage".to_string())),
        }
    }
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

pub fn enumerate_windows_on_api_thread() -> Result<Vec<super::Window>, WindowError> {
    let response = send_command_to_api_thread(WindowsApiCommand::EnumerateWindows)?;
    match response {
        WindowsApiResponse::WindowList(hwnds_raw) => {
            Ok(hwnds_raw.into_iter().map(|handle| super::Window::from_native_impl(WindowsWindow { handle })).collect())
        },
        WindowsApiResponse::Error(e) => Err(e),
        _ => Err(WindowError::ApiError("Unexpected response for EnumerateWindows".to_string())),
    }
}
