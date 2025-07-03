#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use std::{thread, time::Duration, ops::{Sub, Add, Mul}};

use enigo::{
  Button,
  Direction::{Click, Press, Release},
  Enigo, Key as EnigoKey, Keyboard, Mouse, Settings,
};
use napi::{bindgen_prelude::FromNapiValue, JsObject, JsUnknown, ValueType};
use rand::Rng;

pub fn ease_out_quad(t: f64) -> f64 {
  let t_clamped = t.max(0.0).min(1.0);
  t_clamped * (2.0 - t_clamped)
}

pub fn ease_out_cubic(t: f64) -> f64 {
  let t_clamped = t.max(0.0).min(1.0);
  1.0 - (1.0 - t_clamped).powi(3)
}

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

  pub fn magnitude(&self) -> f64 {
    ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
  }

  pub fn interpolate(start: &Position, end: &Position, t: f64) -> Self {
    let one_minus_t = 1.0 - t;
    let control = Position::generate_arc_control_point(start, end, 0.1);

    let x = one_minus_t.powi(2) * (start.x as f64)
      + 2.0 * one_minus_t * t * (control.x as f64)
      + t.powi(2) * (end.x as f64);

    let y = one_minus_t.powi(2) * (start.y as f64)
      + 2.0 * one_minus_t * t * (control.y as f64)
      + t.powi(2) * (end.y as f64);

    Position::new(x.round() as i32, y.round() as i32)
  }

  pub fn generate_arc_control_point(
    start: &Position,
    end: &Position,
    max_arc_magnitude_factor: f64,
  ) -> Self {
    let mut rng = rand::rng();
    let midpoint: Position = &(start + end) * 0.5;
    let difference = end - start;
    let straight_distance = Position::distance(start, end) as f64;

    let perp = if rng.random_bool(0.5) {
      Position::new(-difference.y, difference.x)
    } else {
      Position::new(difference.y, -difference.x)
    };

    let perp_magnitude = perp.magnitude();
    let unit_perp_x = if perp_magnitude != 0.0 {
      (perp.x as f64) / perp_magnitude
    } else {
      0.0
    };
    let unit_perp_y = if perp_magnitude != 0.0 {
      (perp.y as f64) / perp_magnitude
    } else {
      0.0
    };

    let arc_magnitude = rng.random_range(0.0..=(straight_distance * max_arc_magnitude_factor));
    let control_x = (midpoint.x as f64) + unit_perp_x * arc_magnitude;
    let control_y = (midpoint.y as f64) + unit_perp_y * arc_magnitude;

    Position::new(control_x.round() as i32, control_y.round() as i32)
  }

  pub fn distance(position1: &Position, position2: &Position) -> u32 {
    let difference = position1 - position2;
    difference.magnitude() as u32
  }

  pub fn from_polar(angle_turns: f64, magnitude: f64) -> Self {
    let angle_rad = angle_turns * 2.0 * std::f64::consts::PI;

    let x = magnitude * angle_rad.cos();
    let y = magnitude * angle_rad.sin();
    Position::new(x.round() as i32, y.round() as i32)
  }
}

impl Add for &Position {
  type Output = Position;

  fn add(self, rhs: Self) -> Self::Output {
    Position::new(self.x + rhs.x, self.y + rhs.y)
  }
}

impl Sub for &Position {
  type Output = Position;

  fn sub(self, rhs: Self) -> Self::Output {
    Position::new(self.x - rhs.x, self.y - rhs.y)
  }
}

impl Mul<f64> for &Position {
  type Output = Position;

  fn mul(self, rhs: f64) -> Self::Output {
    Position::new(((self.x as f64) * rhs) as i32, ((self.y as f64) * rhs) as i32)
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
  Hangul,
  Hanja,
  Help,
  Home,
  Insert,
  Kanji,
  LControl,
  LeftArrow,
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
  Return,
  RightArrow,
  RShift,
  Select,
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
      SpecialKey::F20 => EnigoKey::F20,
      SpecialKey::F21 => EnigoKey::F21,
      SpecialKey::F22 => EnigoKey::F22,
      SpecialKey::F23 => EnigoKey::F23,
      SpecialKey::F24 => EnigoKey::F24,
      SpecialKey::Hangul => EnigoKey::Hangul,
      SpecialKey::Hanja => EnigoKey::Hanja,
      SpecialKey::Help => EnigoKey::Help,
      SpecialKey::Home => EnigoKey::Home,
      SpecialKey::Insert => EnigoKey::Insert,
      SpecialKey::Kanji => EnigoKey::Kanji,
      SpecialKey::LControl => EnigoKey::LControl,
      SpecialKey::LeftArrow => EnigoKey::LeftArrow,
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
      SpecialKey::Return => EnigoKey::Return,
      SpecialKey::RightArrow => EnigoKey::RightArrow,
      SpecialKey::RShift => EnigoKey::RShift,
      SpecialKey::Select => EnigoKey::Select,
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
  pub fn humanlike_move_mouse(&mut self, x: i32, y: i32, duration: u32) {
    let step = 10;
    let minimum_distance = 50;
    let original_target_position = Position { x, y };
    let mut rng = rand::rng();
    let mouse_position = self.get_mouse_position();
    let mut target_position = Position { x, y };
    let mut adjusted_duration = duration / step;

    let distance = Position::distance(&mouse_position, &target_position);
    if distance > minimum_distance {
      let angle_turns = rng.random_range(0.0..=1.0);
      let magnitude_percentage = rng.random_range(0.0..=0.1);
      let magnitude = distance as f64 * magnitude_percentage;

      target_position = &original_target_position - &Position::from_polar(angle_turns, magnitude);
      adjusted_duration = ((duration as f64 * (1.0 - magnitude_percentage) as f64) as u32) / step;
    }

    for t in 0..(adjusted_duration) {
      let percentage = t as f64 / adjusted_duration as f64;
      let interpolated_position = Position::interpolate(
        &mouse_position,
        &target_position,
        ease_out_cubic(percentage),
      );

      self.move_mouse(interpolated_position.x, interpolated_position.y);
      thread::sleep(Duration::from_millis(step.into()));
    }

    let new_position = self.get_mouse_position();

    if Position::distance(&new_position, &original_target_position) >= 1 {
      self.humanlike_move_mouse(
        original_target_position.x,
        original_target_position.y,
        duration - adjusted_duration,
      );
    }
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
