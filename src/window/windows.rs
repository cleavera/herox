use windows::{
    Win32::Foundation::{BOOL, HWND, LPARAM, RECT, TRUE, FALSE},
    Win32::Graphics::Gdi::{BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, SRCCOPY, GetDIBits, ReleaseDC, CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, BitBlt, DeleteDC, DeleteObject, GetWindowDC},
    Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, GetWindowRect, IsWindowVisible, GetForegroundWindow},
};

use crate::window::{NativeWindow, WindowError};

#[cfg(target_os = "windows")]
pub struct WindowsWindow {
    hwnd: HWND,
}

#[cfg(target_os = "windows")]
impl WindowsWindow {
    fn rect(&self) -> Result<RECT, WindowError> {
        let mut rect = RECT::default();
        if unsafe { GetWindowRect(self.hwnd, &mut rect) } == FALSE {
            Err(WindowError::ApiError("Failed to get window rect".to_string()))
        } else {
            Ok(rect)
        }
    }
}

#[cfg(target_os = "windows")]
impl Clone for WindowsWindow {
    fn clone(&self) -> Self {
        Self { hwnd: self.hwnd }
    }
}

#[cfg(target_os = "windows")]
impl NativeWindow for WindowsWindow {
    fn box_clone(&self) -> Box<dyn NativeWindow + Send + Sync> {
        Box::new(self.clone())
    }

    fn title(&self) -> Result<String, WindowError> {
        let mut text: [u16; 512] = [0; 512];
        let len = unsafe { GetWindowTextW(self.hwnd, &mut text) };
        if len == 0 {
            return Ok("".to_string());
        }
        Ok(String::from_utf16_lossy(&text[..len as usize]))
    }

    fn x(&self) -> Result<i32, WindowError> {
        self.rect().map(|r| r.left)
    }

    fn y(&self) -> Result<i32, WindowError> {
        self.rect().map(|r| r.top)
    }

    fn width(&self) -> Result<u32, WindowError> {
        self.rect().map(|r| (r.right - r.left) as u32)
    }

    fn height(&self) -> Result<u32, WindowError> {
        self.rect().map(|r| (r.bottom - r.top) as u32)
    }

    fn is_focused(&self) -> Result<bool, WindowError> {
        let foreground_window = unsafe { GetForegroundWindow() };
        Ok(self.hwnd == foreground_window)
    }

    fn capture_image(&self) -> Result<image::RgbaImage, WindowError> {
        let rect = self.rect()?;

        let width = (rect.right - rect.left) as i32;
        let height = (rect.bottom - rect.top) as i32;

        let hdc = unsafe { GetWindowDC(self.hwnd) };
        if hdc.0.is_null() {
            return Err(WindowError::ApiError("Failed to get device context".to_string()));
        }

        let mem_dc = unsafe { CreateCompatibleDC(hdc) };
        if mem_dc.0.is_null() {
            unsafe { ReleaseDC(self.hwnd, hdc) };
            return Err(WindowError::ApiError("Failed to create compatible DC".to_string()));
        }

        let mem_bitmap = unsafe { CreateCompatibleBitmap(hdc, width, height) };
        if mem_bitmap.0.is_null() {
            unsafe { DeleteDC(mem_dc) };
            unsafe { ReleaseDC(self.hwnd, hdc) };
            return Err(WindowError::ApiError("Failed to create compatible bitmap".to_string()));
        }

        let old_bitmap = unsafe { SelectObject(mem_dc, mem_bitmap) };

        if unsafe { BitBlt(mem_dc, 0, 0, width, height, hdc, 0, 0, SRCCOPY) } == FALSE {
            unsafe { SelectObject(mem_dc, old_bitmap) };
            unsafe { DeleteObject(mem_bitmap) };
            unsafe { DeleteDC(mem_dc) };
            unsafe { ReleaseDC(self.hwnd, hdc) };
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

        unsafe {
            SelectObject(mem_dc, old_bitmap);
            DeleteObject(mem_bitmap);
            DeleteDC(mem_dc);
            ReleaseDC(self.hwnd, hdc);
        }

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
}

#[cfg(target_os = "windows")]
pub extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<super::Window>) };
    if unsafe { IsWindowVisible(hwnd) } == TRUE {
        let mut text: [u16; 512] = [0; 512];
        let len = unsafe { GetWindowTextW(hwnd, &mut text) };
        if len > 0 {
            windows.push(super::Window::from_native_impl(WindowsWindow { hwnd }));
        }
    }
    TRUE
}