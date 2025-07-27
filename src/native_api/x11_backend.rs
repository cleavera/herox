#![cfg(target_os = "linux")]

use crate::window::WindowError;
use image::RgbaImage;
use once_cell::sync::OnceCell;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Once;
use std::thread;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, ConnectionExt, GetPropertyType, ImageFormat, Window};
use x11rb::rust_connection::RustConnection;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WindowHandle(u32);

impl WindowHandle {
  pub fn new(window: Window) -> Self {
    Self(window)
  }

  pub fn as_window(&self) -> Window {
    self.0
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rect {
  pub left: i32,
  pub top: i32,
  pub right: i32,
  pub bottom: i32,
}

pub enum X11ApiCommand {
  EnumerateWindows,
  GetWindowTitle(WindowHandle),
  GetWindowRect(WindowHandle),
  IsWindowFocused(WindowHandle),
  CaptureWindowImage(WindowHandle),
  Shutdown,
}

pub enum X11ApiResponse {
  WindowList(Vec<WindowHandle>),
  WindowTitle(String),
  WindowRect(Rect),
  WindowFocused(bool),
  WindowImage(RgbaImage),
  Error(WindowError),
  Acknowledgement,
}

fn x11_api_thread_main(receiver: Receiver<(X11ApiCommand, Sender<X11ApiResponse>)>) {
  let (conn, screen_num) = match x11rb::connect(None) {
    Ok(c) => c,
    Err(e) => {
      // We can't send an error back if the connection fails, so we panic.
      panic!("Failed to connect to X11 server: {}", e);
    }
  };

  let screen = &conn.setup().roots[screen_num];
  let root_window = screen.root;

  let net_wm_name = conn
    .intern_atom(false, b"_NET_WM_NAME")
    .unwrap()
    .reply()
    .unwrap()
    .atom;
  let utf8_string = conn
    .intern_atom(false, b"UTF8_STRING")
    .unwrap()
    .reply()
    .unwrap()
    .atom;

  while let Ok((command, response_sender)) = receiver.recv() {
    match command {
      X11ApiCommand::EnumerateWindows => {
        let response = enumerate_windows(&conn, root_window, net_wm_name);
        response_sender.send(response).ok();
      }
      X11ApiCommand::GetWindowTitle(handle) => {
        let response = get_window_title(&conn, handle.as_window(), net_wm_name, utf8_string);
        response_sender.send(response).ok();
      }
      X11ApiCommand::GetWindowRect(handle) => {
        let response = get_window_rect(&conn, root_window, handle.as_window());
        response_sender.send(response).ok();
      }
      X11ApiCommand::IsWindowFocused(handle) => {
        let response = is_window_focused(&conn, handle.as_window());
        response_sender.send(response).ok();
      }
      X11ApiCommand::CaptureWindowImage(handle) => {
        let response = capture_window_image(&conn, handle.as_window());
        response_sender.send(response).ok();
      }
      X11ApiCommand::Shutdown => {
        response_sender.send(X11ApiResponse::Acknowledgement).ok();
        break;
      }
    }
  }
}

fn enumerate_windows(conn: &RustConnection, root: Window, net_wm_name: Atom) -> X11ApiResponse {
  let query_tree_reply = match conn
    .query_tree(root)
    .map_err(|e| e.into())
    .and_then(|c| c.reply())
  {
    Ok(reply) => reply,
    Err(e) => {
      return X11ApiResponse::Error(WindowError::ApiError(format!("QueryTree failed: {}", e)))
    }
  };

  let mut windows = Vec::new();
  for &child in query_tree_reply.children.iter() {
    let attrs = match conn
      .get_window_attributes(child)
      .map_err(|e| e.into())
      .and_then(|c| c.reply())
    {
      Ok(attrs) => attrs,
      _ => continue, // Skip windows we can't get attributes for
    };

    if attrs.map_state == x11rb::protocol::xproto::MapState::VIEWABLE {
      let prop = conn.get_property(false, child, net_wm_name, GetPropertyType::ANY, 0, 1024);
      if let Ok(prop_reply) = prop.map_err(|e| e.into()).and_then(|c| c.reply()) {
        if !prop_reply.value.is_empty() {
          windows.push(WindowHandle::new(child));
        }
      }
    }
  }
  X11ApiResponse::WindowList(windows)
}

fn get_window_title(
  conn: &RustConnection,
  window: Window,
  net_wm_name: Atom,
  utf8_string: Atom,
) -> X11ApiResponse {
  let prop = conn
    .get_property(false, window, net_wm_name, utf8_string, 0, u32::MAX)
    .map_err(|e| e.into())
    .and_then(|c| c.reply());

  match prop {
    Ok(reply) => {
      let title = String::from_utf8(reply.value).unwrap_or_default();
      X11ApiResponse::WindowTitle(title)
    }
    Err(e) => X11ApiResponse::Error(WindowError::ApiError(format!(
      "GetProperty (_NET_WM_NAME) failed: {}",
      e
    ))),
  }
}

fn get_window_rect(conn: &RustConnection, root: Window, window: Window) -> X11ApiResponse {
  let geom = match conn
    .get_geometry(window)
    .map_err(|e| e.into())
    .and_then(|c| c.reply())
  {
    Ok(g) => g,
    Err(e) => {
      return X11ApiResponse::Error(WindowError::ApiError(format!("GetGeometry failed: {}", e)))
    }
  };

  let translated = match conn
    .translate_coordinates(window, root, geom.x, geom.y)
    .map_err(|e| e.into())
    .and_then(|c| c.reply())
  {
    Ok(t) => t,
    Err(e) => {
      return X11ApiResponse::Error(WindowError::ApiError(format!(
        "TranslateCoordinates failed: {}",
        e
      )))
    }
  };

  X11ApiResponse::WindowRect(Rect {
    left: translated.dst_x as i32,
    top: translated.dst_y as i32,
    right: (translated.dst_x + geom.width as i16) as i32,
    bottom: (translated.dst_y + geom.height as i16) as i32,
  })
}

fn is_window_focused(conn: &RustConnection, window: Window) -> X11ApiResponse {
  match conn
    .get_input_focus()
    .map_err(|e| e.into())
    .and_then(|c| c.reply())
  {
    Ok(reply) => X11ApiResponse::WindowFocused(reply.focus == window),
    Err(e) => X11ApiResponse::Error(WindowError::ApiError(format!(
      "GetInputFocus failed: {}",
      e
    ))),
  }
}

fn capture_window_image(conn: &RustConnection, window: Window) -> X11ApiResponse {
  let geom = match conn
    .get_geometry(window)
    .map_err(|e| e.into())
    .and_then(|c| c.reply())
  {
    Ok(g) => g,
    Err(e) => {
      return X11ApiResponse::Error(WindowError::ApiError(format!("GetGeometry failed: {}", e)))
    }
  };

  let img = match conn
    .get_image(
      ImageFormat::Z_PIXMAP,
      window,
      0,
      0,
      geom.width,
      geom.height,
      u32::MAX,
    )
    .map_err(|e| e.into())
    .and_then(|c| c.reply())
  {
    Ok(img) => img,
    Err(e) => {
      return X11ApiResponse::Error(WindowError::ApiError(format!("GetImage failed: {}", e)))
    }
  };

  let mut data = img.data;
  for chunk in data.chunks_mut(4) {
    // X11 gives BGRA, we need RGBA
    chunk.swap(0, 2);
  }

  match RgbaImage::from_raw(geom.width as u32, geom.height as u32, data) {
    Some(image) => X11ApiResponse::WindowImage(image),
    None => X11ApiResponse::Error(WindowError::InvalidBitmap),
  }
}

static X11_API_SENDER: OnceCell<Sender<(X11ApiCommand, Sender<X11ApiResponse>)>> = OnceCell::new();
static INIT_X11_API_THREAD: Once = Once::new();

pub fn send_command_to_api_thread(command: X11ApiCommand) -> Result<X11ApiResponse, WindowError> {
  INIT_X11_API_THREAD.call_once(|| {
    let (sender, receiver) = channel();
    X11_API_SENDER.set(sender).unwrap();
    thread::spawn(move || x11_api_thread_main(receiver));
  });

  let (response_sender, response_receiver) = channel();
  let sender = X11_API_SENDER.get().unwrap();
  sender
    .send((command, response_sender))
    .map_err(|e| WindowError::ApiError(format!("Failed to send command to API thread: {}", e)))?;
  Ok(response_receiver.recv().map_err(|e| {
    WindowError::ApiError(format!("Failed to receive response from API thread: {}", e))
  })?)
}
