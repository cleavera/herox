use enigo::{Enigo, InputError, Settings};
use napi::{Error, Task, JsError};
use xcap::{Monitor, XCapError};

use std::{thread, time::Duration};

use enigo::{Button as EnigoButton, Direction::Click, Mouse as EnigoMouse};
use rand::Rng;

use crate::position::Position;

pub fn ease_out_quad(t: f64) -> f64 {
  let t_clamped = t.max(0.0).min(1.0);
  t_clamped * (2.0 - t_clamped)
}

pub fn ease_out_cubic(t: f64) -> f64 {
  let t_clamped = t.max(0.0).min(1.0);
  1.0 - (1.0 - t_clamped).powi(3)
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

impl From<EnigoButton> for MouseButton {
  fn from(value: EnigoButton) -> Self {
    match value {
      EnigoButton::Left => MouseButton::Left,
      EnigoButton::Middle => MouseButton::Middle,
      EnigoButton::Right => MouseButton::Right,
      EnigoButton::Back => MouseButton::Back,
      EnigoButton::Forward => MouseButton::Forward,
      EnigoButton::ScrollUp => MouseButton::ScrollUp,
      EnigoButton::ScrollDown => MouseButton::ScrollDown,
      EnigoButton::ScrollLeft => MouseButton::ScrollLeft,
      EnigoButton::ScrollRight => MouseButton::ScrollRight,
    }
  }
}

impl Into<EnigoButton> for MouseButton {
  fn into(self) -> EnigoButton {
    match self {
      MouseButton::Left => EnigoButton::Left,
      MouseButton::Middle => EnigoButton::Middle,
      MouseButton::Right => EnigoButton::Right,
      MouseButton::Back => EnigoButton::Back,
      MouseButton::Forward => EnigoButton::Forward,
      MouseButton::ScrollUp => EnigoButton::ScrollUp,
      MouseButton::ScrollDown => EnigoButton::ScrollDown,
      MouseButton::ScrollLeft => EnigoButton::ScrollLeft,
      MouseButton::ScrollRight => EnigoButton::ScrollRight,
    }
  }
}

pub struct MouseError {
  message: String,
}

impl From<MouseError> for Error {
  fn from(value: MouseError) -> Error {
    Error::from_reason(value.message)
  }
}

impl From<InputError> for MouseError {
  fn from(value: InputError) -> Self {
    MouseError {
      message: value.to_string(),
    }
  }
}

impl From<XCapError> for MouseError {
  fn from(value: XCapError) -> Self {
    MouseError {
      message: value.to_string(),
    }
  }
}

#[napi]
pub struct Mouse {
  enigo: Enigo,
}

#[napi]
impl Mouse {
  #[napi(constructor)]
  pub fn new() -> Self {
    Mouse {
      enigo: Enigo::new(&Settings::default()).unwrap(),
    }
  }

  #[napi]
  pub fn get_position(&self) -> Result<Position, Error> {
    Ok(self.enigo.location().map_err(MouseError::from)?.into())
  }

  #[napi]
  pub fn move_to(&mut self, x: i32, y: i32) -> Result<(), Error> {
    self
      .enigo
      .move_mouse(x.max(0), y.max(0), enigo::Coordinate::Abs)
      .map_err(MouseError::from)?;

    Ok(())
  }

  #[napi]
  pub fn humanlike_move_to(&mut self, x: i32, y: i32, duration: u32) -> Result<(), Error> {
    let step = 10;
    let minimum_distance = 50;
    let original_target_position = Position { x, y };
    let mut rng = rand::rng();
    let mouse_position = self.get_position()?;
    let mut target_position = Position { x, y };
    let mut adjusted_duration = duration / step;
    let monitors = Monitor::all().map_err(MouseError::from)?;
    let monitor = monitors.first().expect("No monitor found");

    let min_pos = Position::new(0, 0);

    let max_pos = &min_pos
      + &Position::new(
        monitor.width().map_err(MouseError::from)? as i32,
        monitor.height().map_err(MouseError::from)? as i32,
      );

    let distance = Position::distance(&mouse_position, &target_position);
    if distance > minimum_distance {
      let angle_turns = rng.random_range(0.0..=1.0);
      let magnitude_percentage = rng.random_range(0.0..=0.1);
      let magnitude = distance as f64 * magnitude_percentage;

      target_position = (&original_target_position - &Position::from_polar(angle_turns, magnitude))
        .clamp(&min_pos, &max_pos);
      adjusted_duration = ((duration as f64 * (1.0 - magnitude_percentage) as f64) as u32) / step;
    }

    let control_point =
      Position::generate_arc_control_point(&mouse_position, &target_position, 0.1);

    for t in 0..(adjusted_duration) {
      let percentage = t as f64 / adjusted_duration as f64;
      let interpolated_position = Position::interpolate(
        &mouse_position,
        &target_position,
        &control_point,
        ease_out_cubic(percentage),
      );

      self.move_to(interpolated_position.x, interpolated_position.y)?;
      thread::sleep(Duration::from_millis(step.into()));
    }

    let new_position = self.get_position()?;

    if Position::distance(&new_position, &original_target_position) >= 1 {
      self.humanlike_move_to(
        original_target_position.x,
        original_target_position.y,
        duration - adjusted_duration,
      )?;
    }

    Ok(())
  }

  #[napi]
  pub fn click(&mut self, button: MouseButton) -> Result<(), Error> {
    self
      .enigo
      .button(button.into(), Click)
      .map_err(MouseError::from)?;

    Ok(())
  }
}
