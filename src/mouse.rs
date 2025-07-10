use enigo::{Enigo, Settings};

use std::{thread, time::Duration};

use enigo::{
  Button as EnigoButton,
  Direction::Click, Mouse as EnigoMouse,
};
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
  pub fn get_position(&self) -> Position {
    self.enigo.location().unwrap().into()
  }

  #[napi]
  pub fn move_to(&mut self, x: i32, y: i32) {
    self.enigo.move_mouse(x, y, enigo::Coordinate::Abs).unwrap();
  }

  #[napi]
  pub fn humanlike_move_to(&mut self, x: i32, y: i32, duration: u32) {
    let step = 10;
    let minimum_distance = 50;
    let original_target_position = Position { x, y };
    let mut rng = rand::rng();
    let mouse_position = self.get_position();
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

      self.move_to(interpolated_position.x, interpolated_position.y);
      thread::sleep(Duration::from_millis(step.into()));
    }

    let new_position = self.get_position();

    if Position::distance(&new_position, &original_target_position) >= 1 {
      self.humanlike_move_to(
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
}
