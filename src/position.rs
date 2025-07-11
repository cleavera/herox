use std::ops::{Add, Mul, Sub};
use rand::Rng;

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

  pub fn interpolate(start: &Position, end: &Position, control_point: &Position, t: f64) -> Self {
    let one_minus_t = 1.0 - t;

    let x = one_minus_t.powi(2) * (start.x as f64)
      + 2.0 * one_minus_t * t * (control_point.x as f64)
      + t.powi(2) * (end.x as f64);

    let y = one_minus_t.powi(2) * (start.y as f64)
      + 2.0 * one_minus_t * t * (control_point.y as f64)
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

  pub fn clamp(&self, min: &Position, max: &Position) -> Self {
    Position::new(
      self.x.clamp(min.x, max.x),
      self.y.clamp(min.y, max.y),
    )
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
