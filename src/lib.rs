#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use enigo::{
  Button,
  Direction::{Click, Press, Release},
  Enigo, Key as EnigoKey, Keyboard, Mouse, Settings,
};
use napi::{bindgen_prelude::FromNapiValue, JsObject, JsUnknown, ValueType};

#[napi]
pub struct Position {
  pub x: i32,
  pub y: i32,
}

#[napi]
impl Position {
  #[napi(constructor)]
  pub fn new(x: i32, y: i32) -> Self {
    Position { x, y }
  }
}

impl From<(i32, i32)> for Position {
  fn from(value: (i32, i32)) -> Self {
    Position::new(value.0, value.1)
  }
}

#[napi(string_enum)]
pub enum MouseButton {
  Left,
  Middle,
  Right,
  Back,
  Forward,
  ScrollUp,
  ScrollDown,
  ScrollLeft,
  ScrollRight,
}

impl From<Button> for MouseButton {
  fn from(value: Button) -> Self {
    match value {
      Button::Left => MouseButton::Left,
      Button::Middle => MouseButton::Middle,
      Button::Right => MouseButton::Right,
      Button::Back => MouseButton::Back,
      Button::Forward => MouseButton::Forward,
      Button::ScrollUp => MouseButton::ScrollUp,
      Button::ScrollDown => MouseButton::ScrollDown,
      Button::ScrollLeft => MouseButton::ScrollLeft,
      Button::ScrollRight => MouseButton::ScrollRight,
    }
  }
}

impl Into<Button> for MouseButton {
  fn into(self) -> Button {
    match self {
      MouseButton::Left => Button::Left,
      MouseButton::Middle => Button::Middle,
      MouseButton::Right => Button::Right,
      MouseButton::Back => Button::Back,
      MouseButton::Forward => Button::Forward,
      MouseButton::ScrollUp => Button::ScrollUp,
      MouseButton::ScrollDown => Button::ScrollDown,
      MouseButton::ScrollLeft => Button::ScrollLeft,
      MouseButton::ScrollRight => Button::ScrollRight,
    }
  }
}

#[napi(string_enum)]
pub enum SpecialKey {
  Add,
  Alt,
  Backspace,
  Break,
  Begin,
  Cancel,
  CapsLock,
  Clear,
  Command,
  Control,
  Decimal,
  Delete,
  Divide,
  DownArrow,
  End,
  Escape,
  Execute,
  F1,
  F2,
  F3,
  F4,
  F5,
  F6,
  F7,
  F8,
  F9,
  F10,
  F11,
  F12,
  F13,
  F14,
  F15,
  F16,
  F17,
  F18,
  F19,
  F20,
  F21,
  F22,
  F23,
  F24,
  F25,
  F26,
  F27,
  F28,
  F29,
  F30,
  F31,
  F32,
  F33,
  F34,
  F35,
  Find,
  Hangul,
  Hanja,
  Help,
  Home,
  Insert,
  Kanji,
  LControl,
  LeftArrow,
  Linefeed,
  LMenu,
  LShift,
  MediaNextTrack,
  MediaPlayPause,
  MediaPrevTrack,
  MediaStop,
  Meta,
  ModeChange,
  Multiply,
  Numlock,
  Numpad0,
  Numpad1,
  Numpad2,
  Numpad3,
  Numpad4,
  Numpad5,
  Numpad6,
  Numpad7,
  Numpad8,
  Numpad9,
  Option,
  PageDown,
  PageUp,
  Pause,
  Print,
  PrintScr,
  RControl,
  Redo,
  Return,
  RightArrow,
  RShift,
  ScrollLock,
  Select,
  ScriptSwitch,
  Shift,
  ShiftLock,
  Space,
  Subtract,
  SysReq,
  Tab,
  Undo,
  UpArrow,
  VolumeDown,
  VolumeMute,
  VolumeUp,
  MicMute,
}

pub enum Key {
  Special(SpecialKey),
  Unicode(char),
}

#[napi(object)]
pub struct UnicodeKey {
  pub kind: String,
  pub value: String,
}

#[napi]
pub fn unicode(s: String) -> UnicodeKey {
  UnicodeKey {
    kind: "Unicode".to_owned(),
    value: s,
  }
}

impl Into<EnigoKey> for SpecialKey {
  fn into(self) -> EnigoKey {
    match self {
      SpecialKey::Add => EnigoKey::Add,
      SpecialKey::Alt => EnigoKey::Alt,
      SpecialKey::Backspace => EnigoKey::Backspace,
      SpecialKey::Break => EnigoKey::Break,
      SpecialKey::Begin => EnigoKey::Begin,
      SpecialKey::Cancel => EnigoKey::Cancel,
      SpecialKey::CapsLock => EnigoKey::CapsLock,
      SpecialKey::Clear => EnigoKey::Clear,
      SpecialKey::Command => EnigoKey::Meta,
      SpecialKey::Control => EnigoKey::Control,
      SpecialKey::Decimal => EnigoKey::Decimal,
      SpecialKey::Delete => EnigoKey::Delete,
      SpecialKey::Divide => EnigoKey::Divide,
      SpecialKey::DownArrow => EnigoKey::DownArrow,
      SpecialKey::End => EnigoKey::End,
      SpecialKey::Escape => EnigoKey::Escape,
      SpecialKey::Execute => EnigoKey::Execute,
      SpecialKey::F1 => EnigoKey::F1,
      SpecialKey::F2 => EnigoKey::F2,
      SpecialKey::F3 => EnigoKey::F3,
      SpecialKey::F4 => EnigoKey::F4,
      SpecialKey::F5 => EnigoKey::F5,
      SpecialKey::F6 => EnigoKey::F6,
      SpecialKey::F7 => EnigoKey::F7,
      SpecialKey::F8 => EnigoKey::F8,
      SpecialKey::F9 => EnigoKey::F9,
      SpecialKey::F10 => EnigoKey::F10,
      SpecialKey::F11 => EnigoKey::F11,
      SpecialKey::F12 => EnigoKey::F12,
      SpecialKey::F13 => EnigoKey::F13,
      SpecialKey::F14 => EnigoKey::F14,
      SpecialKey::F15 => EnigoKey::F15,
      SpecialKey::F16 => EnigoKey::F16,
      SpecialKey::F17 => EnigoKey::F17,
      SpecialKey::F18 => EnigoKey::F18,
      SpecialKey::F19 => EnigoKey::F19,
      SpecialKey::F20 => EnigoKey::F20,
      SpecialKey::F21 => EnigoKey::F21,
      SpecialKey::F22 => EnigoKey::F22,
      SpecialKey::F23 => EnigoKey::F23,
      SpecialKey::F24 => EnigoKey::F24,
      SpecialKey::F25 => EnigoKey::F25,
      SpecialKey::F26 => EnigoKey::F26,
      SpecialKey::F27 => EnigoKey::F27,
      SpecialKey::F28 => EnigoKey::F28,
      SpecialKey::F29 => EnigoKey::F29,
      SpecialKey::F30 => EnigoKey::F30,
      SpecialKey::F31 => EnigoKey::F31,
      SpecialKey::F32 => EnigoKey::F32,
      SpecialKey::F33 => EnigoKey::F33,
      SpecialKey::F34 => EnigoKey::F34,
      SpecialKey::F35 => EnigoKey::F35,
      SpecialKey::Find => EnigoKey::Find,
      SpecialKey::Hangul => EnigoKey::Hangul,
      SpecialKey::Hanja => EnigoKey::Hanja,
      SpecialKey::Help => EnigoKey::Help,
      SpecialKey::Home => EnigoKey::Home,
      SpecialKey::Insert => EnigoKey::Insert,
      SpecialKey::Kanji => EnigoKey::Kanji,
      SpecialKey::LControl => EnigoKey::LControl,
      SpecialKey::LeftArrow => EnigoKey::LeftArrow,
      SpecialKey::Linefeed => EnigoKey::Linefeed,
      SpecialKey::LMenu => EnigoKey::LMenu,
      SpecialKey::LShift => EnigoKey::LShift,
      SpecialKey::MediaNextTrack => EnigoKey::MediaNextTrack,
      SpecialKey::MediaPlayPause => EnigoKey::MediaPlayPause,
      SpecialKey::MediaPrevTrack => EnigoKey::MediaPrevTrack,
      SpecialKey::MediaStop => EnigoKey::MediaStop,
      SpecialKey::Meta => EnigoKey::Meta,
      SpecialKey::ModeChange => EnigoKey::ModeChange,
      SpecialKey::Multiply => EnigoKey::Multiply,
      SpecialKey::Numlock => EnigoKey::Numlock,
      SpecialKey::Numpad0 => EnigoKey::Numpad0,
      SpecialKey::Numpad1 => EnigoKey::Numpad1,
      SpecialKey::Numpad2 => EnigoKey::Numpad2,
      SpecialKey::Numpad3 => EnigoKey::Numpad3,
      SpecialKey::Numpad4 => EnigoKey::Numpad4,
      SpecialKey::Numpad5 => EnigoKey::Numpad5,
      SpecialKey::Numpad6 => EnigoKey::Numpad6,
      SpecialKey::Numpad7 => EnigoKey::Numpad7,
      SpecialKey::Numpad8 => EnigoKey::Numpad8,
      SpecialKey::Numpad9 => EnigoKey::Numpad9,
      SpecialKey::Option => EnigoKey::Option,
      SpecialKey::PageDown => EnigoKey::PageDown,
      SpecialKey::PageUp => EnigoKey::PageUp,
      SpecialKey::Pause => EnigoKey::Pause,
      SpecialKey::Print => EnigoKey::PrintScr,
      SpecialKey::PrintScr => EnigoKey::PrintScr,
      SpecialKey::RControl => EnigoKey::RControl,
      SpecialKey::Redo => EnigoKey::Redo,
      SpecialKey::Return => EnigoKey::Return,
      SpecialKey::RightArrow => EnigoKey::RightArrow,
      SpecialKey::RShift => EnigoKey::RShift,
      SpecialKey::ScrollLock => EnigoKey::ScrollLock,
      SpecialKey::Select => EnigoKey::Select,
      SpecialKey::ScriptSwitch => EnigoKey::ScriptSwitch,
      SpecialKey::Shift => EnigoKey::Shift,
      SpecialKey::ShiftLock => EnigoKey::ShiftLock,
      SpecialKey::Space => EnigoKey::Space,
      SpecialKey::Subtract => EnigoKey::Subtract,
      SpecialKey::SysReq => EnigoKey::SysReq,
      SpecialKey::Tab => EnigoKey::Tab,
      SpecialKey::Undo => EnigoKey::Undo,
      SpecialKey::UpArrow => EnigoKey::UpArrow,
      SpecialKey::VolumeDown => EnigoKey::VolumeDown,
      SpecialKey::VolumeMute => EnigoKey::VolumeMute,
      SpecialKey::VolumeUp => EnigoKey::VolumeUp,
      SpecialKey::MicMute => EnigoKey::MicMute,
    }
  }
}

#[napi]
pub struct Herox {
  enigo: Enigo,
}

#[napi]
impl Herox {
  #[napi(constructor)]
  pub fn new() -> Self {
    Herox {
      enigo: Enigo::new(&Settings::default()).unwrap(),
    }
  }

  #[napi]
  pub fn get_mouse_position(&self) -> Position {
    self.enigo.location().unwrap().into()
  }

  #[napi]
  pub fn move_mouse(&mut self, x: i32, y: i32) {
    self.enigo.move_mouse(x, y, enigo::Coordinate::Abs).unwrap();
  }

  #[napi]
  pub fn click(&mut self, button: MouseButton) {
    self.enigo.button(button.into(), Click).unwrap();
  }

  #[napi(ts_args_type = "key: UnicodeKey | SpecialKey")]
  pub fn key_down(&mut self, key: JsUnknown) {
    self.enigo.key(Self::get_key(key).unwrap(), Press).unwrap();
  }

  #[napi(ts_args_type = "key: UnicodeKey | SpecialKey")]
  pub fn key_up(&mut self, key: JsUnknown) {
    self
      .enigo
      .key(Self::get_key(key).unwrap(), Release)
      .unwrap();
  }

  #[napi(ts_args_type = "key: UnicodeKey | SpecialKey")]
  pub fn key_press(&mut self, key: JsUnknown) {
    self.enigo.key(Self::get_key(key).unwrap(), Click).unwrap();
  }

  fn get_key(arg: JsUnknown) -> Result<EnigoKey, napi::Error> {
    match arg.get_type()? {
      ValueType::String => {
        let key: SpecialKey = SpecialKey::from_unknown(arg)?;

        Ok(key.into())
      }
      ValueType::Object => {
        let obj: JsObject = arg.try_into()?;
        let kind: String = obj.get_named_property("kind")?;
        if kind == "Unicode" {
          let value: String = obj.get_named_property("value")?;
          let mut chars = value.chars();
          let ch = chars
            .next()
            .ok_or_else(|| napi::Error::from_reason("Empty char"))?;
          if chars.next().is_some() {
            return Err(napi::Error::from_reason(
              "Unicode value must be a single char",
            ));
          }
          let key = EnigoKey::Unicode(ch);
          Ok(key)
        } else {
          Err(napi::Error::from_reason("Unknown object kind"))
        }
      }
      _ => Err(napi::Error::from_reason("Invalid argument type")),
    }
  }
}
