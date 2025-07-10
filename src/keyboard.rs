use enigo::{
  Direction::{Click, Press, Release},
  Enigo, Key as EnigoKey, Keyboard as EnigoKeyboard, Settings, InputError,
};
use napi::{bindgen_prelude::FromNapiValue, Error, JsObject, JsUnknown, ValueType};

#[napi(string_enum)]
pub enum SpecialKey {
  Add,
  Alt,
  Backspace,
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
  Help,
  Home,
  #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
  Insert,
  LControl,
  LeftArrow,
  LShift,
  MediaNextTrack,
  MediaPlayPause,
  MediaPrevTrack,
  Meta,
  Multiply,
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
  RControl,
  Return,
  RightArrow,
  RShift,
  Shift,
  Space,
  Subtract,
  Tab,
  UpArrow,
  VolumeDown,
  VolumeMute,
  VolumeUp,
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
      SpecialKey::Help => EnigoKey::Help,
      SpecialKey::Home => EnigoKey::Home,
      SpecialKey::Insert => EnigoKey::Insert,
      SpecialKey::LControl => EnigoKey::LControl,
      SpecialKey::LeftArrow => EnigoKey::LeftArrow,
      SpecialKey::LShift => EnigoKey::LShift,
      SpecialKey::MediaNextTrack => EnigoKey::MediaNextTrack,
      SpecialKey::MediaPlayPause => EnigoKey::MediaPlayPause,
      SpecialKey::MediaPrevTrack => EnigoKey::MediaPrevTrack,
      SpecialKey::Meta => EnigoKey::Meta,
      SpecialKey::Multiply => EnigoKey::Multiply,
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
      SpecialKey::RControl => EnigoKey::RControl,
      SpecialKey::Return => EnigoKey::Return,
      SpecialKey::RightArrow => EnigoKey::RightArrow,
      SpecialKey::RShift => EnigoKey::RShift,
      SpecialKey::Shift => EnigoKey::Shift,
      SpecialKey::Space => EnigoKey::Space,
      SpecialKey::Subtract => EnigoKey::Subtract,
      SpecialKey::Tab => EnigoKey::Tab,
      SpecialKey::UpArrow => EnigoKey::UpArrow,
      SpecialKey::VolumeDown => EnigoKey::VolumeDown,
      SpecialKey::VolumeMute => EnigoKey::VolumeMute,
      SpecialKey::VolumeUp => EnigoKey::VolumeUp,
    }
  }
}

pub struct KeyboardError {
    message: String,
}

impl From<KeyboardError> for Error {
    fn from(value: KeyboardError) -> Error {
        Error::from_reason(value.message)
    }
}

impl From<InputError> for KeyboardError {
    fn from(value: InputError) -> Self {
        KeyboardError {
            message: value.to_string(),
        }
    }
}

#[napi]
pub struct Keyboard {
  enigo: Enigo,
}

#[napi]
impl Keyboard {
  #[napi(constructor)]
  pub fn new() -> Self {
    let enigo = Enigo::new(&Settings::default()).unwrap();

    Keyboard { enigo }
  }

  #[napi(ts_args_type = "key: UnicodeKey | SpecialKey")]
  pub fn key_down(&mut self, key: JsUnknown) -> Result<(), Error> {
    self.enigo.key(Self::get_key(key)?, Press).map_err(KeyboardError::from)?;

    Ok(())
  }

  #[napi(ts_args_type = "key: UnicodeKey | SpecialKey")]
  pub fn key_up(&mut self, key: JsUnknown) -> Result<(), Error> {
    self
      .enigo
      .key(Self::get_key(key)?, Release).map_err(KeyboardError::from)?;

    Ok(())
  }

  #[napi(ts_args_type = "key: UnicodeKey | SpecialKey")]
  pub fn key_press(&mut self, key: JsUnknown) -> Result<(), Error> {
    self.enigo.key(Self::get_key(key)?, Click).map_err(KeyboardError::from)?;

    Ok(())
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
