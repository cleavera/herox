#![cfg(target_os = "linux")]

use image::RgbaImage;
use once_cell::sync::OnceCell;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Once;
use std::thread;
use x11rb::connection::Connection;
use x11rb::errors::{ConnectionError, ReplyError};
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

#[derive(Debug, Clone)]
pub enum X11ApiEnumerateWindowsError {
  QueryTreeConnectionError(String),
  QueryTreeReplyError(String),
  Generic(String),
}

impl From<ConnectionError> for X11ApiEnumerateWindowsError {
  fn from(value: ConnectionError) -> Self {
    X11ApiEnumerateWindowsError::QueryTreeConnectionError(value.to_string())
  }
}

impl From<ReplyError> for X11ApiEnumerateWindowsError {
  fn from(value: ReplyError) -> Self {
    X11ApiEnumerateWindowsError::QueryTreeReplyError(value.to_string())
  }
}

#[derive(Debug, Clone)]
pub enum X11ApiGetWindowTitleError {
  GetPropertyConnectionError(String),
  GetPropertyReplyError(String),
  Generic(String),
}

impl From<ConnectionError> for X11ApiGetWindowTitleError {
  fn from(value: ConnectionError) -> Self {
    X11ApiGetWindowTitleError::GetPropertyConnectionError(value.to_string())
  }
}

impl From<ReplyError> for X11ApiGetWindowTitleError {
  fn from(value: ReplyError) -> Self {
    X11ApiGetWindowTitleError::GetPropertyReplyError(value.to_string())
  }
}

#[derive(Debug, Clone)]
pub enum X11ApiGetWindowRectError {
  ConnectionError(String),
  ReplyError(String),
  Generic(String),
}

impl From<ConnectionError> for X11ApiGetWindowRectError {
  fn from(value: ConnectionError) -> Self {
    X11ApiGetWindowRectError::ConnectionError(value.to_string())
  }
}

impl From<ReplyError> for X11ApiGetWindowRectError {
  fn from(value: ReplyError) -> Self {
    X11ApiGetWindowRectError::ReplyError(value.to_string())
  }
}

#[derive(Debug, Clone)]
pub enum X11ApiIsWindowFocusedError {
  ConnectionError(String),
  ReplyError(String),
  Generic(String),
}

impl From<ConnectionError> for X11ApiIsWindowFocusedError {
  fn from(value: ConnectionError) -> Self {
    X11ApiIsWindowFocusedError::ConnectionError(value.to_string())
  }
}

impl From<ReplyError> for X11ApiIsWindowFocusedError {
  fn from(value: ReplyError) -> Self {
    X11ApiIsWindowFocusedError::ReplyError(value.to_string())
  }
}

#[derive(Debug, Clone)]
pub enum X11ApiCaptureWindowImageError {
  ConnectionError(String),
  ReplyError(String),
  InvalidBitmap,
  Generic(String),
}

impl From<ConnectionError> for X11ApiCaptureWindowImageError {
  fn from(value: ConnectionError) -> Self {
    X11ApiCaptureWindowImageError::ConnectionError(value.to_string())
  }
}

impl From<ReplyError> for X11ApiCaptureWindowImageError {
  fn from(value: ReplyError) -> Self {
    X11ApiCaptureWindowImageError::ReplyError(value.to_string())
  }
}

#[derive(Debug, Clone)]
pub enum X11ApiError {
  EnumerateWindows(X11ApiEnumerateWindowsError),
  GetWindowTitle(X11ApiGetWindowTitleError),
  GetWindowRect(X11ApiGetWindowRectError),
  IsWindowFocused(X11ApiIsWindowFocusedError),
  CaptureWindowImage(X11ApiCaptureWindowImageError),
}

pub enum X11ApiResponse {
  WindowList(Vec<WindowHandle>),
  WindowTitle(String),
  WindowRect(Rect),
  WindowFocused(bool),
  WindowImage(RgbaImage),
  Error(X11ApiError),
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
        let response = match enumerate_windows(&conn, root_window, net_wm_name) {
          Ok(windows) => X11ApiResponse::WindowList(windows),
          Err(e) => X11ApiResponse::Error(X11ApiError::EnumerateWindows(e)),
        };
        response_sender.send(response).ok();
      }
      X11ApiCommand::GetWindowTitle(handle) => {
        let response = match get_window_title(&conn, handle.as_window(), net_wm_name, utf8_string) {
          Ok(title) => X11ApiResponse::WindowTitle(title),
          Err(e) => X11ApiResponse::Error(X11ApiError::GetWindowTitle(e)),
        };
        response_sender.send(response).ok();
      }
      X11ApiCommand::GetWindowRect(handle) => {
        let response = match get_window_rect(&conn, root_window, handle.as_window()) {
          Ok(rect) => X11ApiResponse::WindowRect(rect),
          Err(e) => X11ApiResponse::Error(X11ApiError::GetWindowRect(e)),
        };
        response_sender.send(response).ok();
      }
      X11ApiCommand::IsWindowFocused(handle) => {
        let response = match is_window_focused(&conn, handle.as_window()) {
          Ok(focused) => X11ApiResponse::WindowFocused(focused),
          Err(e) => X11ApiResponse::Error(X11ApiError::IsWindowFocused(e)),
        };
        response_sender.send(response).ok();
      }
      X11ApiCommand::CaptureWindowImage(handle) => {
        let response = match capture_window_image(&conn, handle.as_window()) {
          Ok(img) => X11ApiResponse::WindowImage(img),
          Err(e) => X11ApiResponse::Error(X11ApiError::CaptureWindowImage(e)),
        };
        response_sender.send(response).ok();
      }
      X11ApiCommand::Shutdown => {
        response_sender.send(X11ApiResponse::Acknowledgement).ok();
        break;
      }
    }
  }
}

fn enumerate_windows(
  conn: &RustConnection,
  root: Window,
  net_wm_name: Atom,
) -> Result<Vec<WindowHandle>, X11ApiEnumerateWindowsError> {
  let query_tree_reply = conn.query_tree(root)?.reply()?;

  let mut windows = Vec::new();
  for &child in query_tree_reply.children.iter() {
    let attrs = match conn
      .get_window_attributes(child)
      .map_err(|e| e.to_string())
      .and_then(|c| c.reply().map_err(|e| e.to_string()))
    {
      Ok(attrs) => attrs,
      _ => continue, // Skip windows we can't get attributes for
    };

    if attrs.map_state == x11rb::protocol::xproto::MapState::VIEWABLE {
      let prop = conn.get_property(false, child, net_wm_name, GetPropertyType::ANY, 0, 1024);
      if let Ok(prop_reply) = prop
        .map_err(|e| e.to_string())
        .and_then(|c| c.reply().map_err(|e| e.to_string()))
      {
        if !prop_reply.value.is_empty() {
          windows.push(WindowHandle::new(child));
        }
      }
    }
  }
  Ok(windows)
}

fn get_window_title(
  conn: &RustConnection,
  window: Window,
  net_wm_name: Atom,
  utf8_string: Atom,
) -> Result<String, X11ApiGetWindowTitleError> {
  let prop = conn
    .get_property(false, window, net_wm_name, utf8_string, 0, u32::MAX)?
    .reply()?;

  Ok(String::from_utf8(prop.value).unwrap_or_default())
}

fn get_window_rect(
  conn: &RustConnection,
  root: Window,
  window: Window,
) -> Result<Rect, X11ApiGetWindowRectError> {
  let geom = conn.get_geometry(window)?.reply()?;
  let translated = conn
    .translate_coordinates(window, root, geom.x, geom.y)?
    .reply()?;

  Ok(Rect {
    left: translated.dst_x as i32,
    top: translated.dst_y as i32,
    right: (translated.dst_x + geom.width as i16) as i32,
    bottom: (translated.dst_y + geom.height as i16) as i32,
  })
}

fn is_window_focused(
  conn: &RustConnection,
  window: Window,
) -> Result<bool, X11ApiIsWindowFocusedError> {
  let reply = conn.get_input_focus()?.reply()?;

  Ok(reply.focus == window)
}

fn capture_window_image(
  conn: &RustConnection,
  window: Window,
) -> Result<RgbaImage, X11ApiCaptureWindowImageError> {
  let geom = conn
    .get_geometry(window)?.reply()?;

  let img = conn
    .get_image(
      ImageFormat::Z_PIXMAP,
      window,
      0,
      0,
      geom.width,
      geom.height,
      u32::MAX,
    )?.reply()?;

  let mut data = img.data;
  for chunk in data.chunks_mut(4) {
    // X11 gives BGRA, we need RGBA
    chunk.swap(0, 2);
  }

  RgbaImage::from_raw(geom.width as u32, geom.height as u32, data)
    .ok_or_else(|| X11ApiCaptureWindowImageError::InvalidBitmap)
}

static X11_API_SENDER: OnceCell<Sender<(X11ApiCommand, Sender<X11ApiResponse>)>> = OnceCell::new();
static INIT_X11_API_THREAD: Once = Once::new();

#[derive(Clone, Copy, Debug)]
pub enum X11SendCommandToApiThreadError {
  Send,
  Receive,
}

pub fn send_command_to_api_thread(
  command: X11ApiCommand,
) -> Result<X11ApiResponse, X11SendCommandToApiThreadError> {
  INIT_X11_API_THREAD.call_once(|| {
    let (sender, receiver) = channel();
    X11_API_SENDER.set(sender).unwrap();
    thread::spawn(move || x11_api_thread_main(receiver));
  });

  let (response_sender, response_receiver) = channel();
  let sender = X11_API_SENDER.get().unwrap();
  sender
    .send((command, response_sender))
    .map_err(|_| X11SendCommandToApiThreadError::Send)?;
  Ok(
    response_receiver
      .recv()
      .map_err(|_| X11SendCommandToApiThreadError::Receive)?,
  )
}
