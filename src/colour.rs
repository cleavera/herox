use image::Rgba;

pub fn rgba_into_rgba_number(rgba: &Rgba<u8>) -> u32 {
  ((rgba.0[0] as u32) << 24)
    | ((rgba.0[1] as u32) << 16)
    | ((rgba.0[2] as u32) << 8)
    | (rgba.0[3] as u32)
}

pub fn rgba_number_into_rgba(rgba_number: u32) -> Rgba<u8> {
  Rgba([
    ((rgba_number >> 24) & 0xFF) as u8,
    ((rgba_number >> 16) & 0xFF) as u8,
    ((rgba_number >> 8) & 0xFF) as u8,
    (rgba_number & 0xFF) as u8,
  ])
}

pub fn color_distance(color1_u32: u32, color2_u32: u32, use_alpha: bool) -> f64 {
  let rgba1 = rgba_number_into_rgba(color1_u32);
  let rgba2 = rgba_number_into_rgba(color2_u32);

  let dr = (rgba1.0[0] as f64) - (rgba2.0[0] as f64);
  let dg = (rgba1.0[1] as f64) - (rgba2.0[1] as f64);
  let db = (rgba1.0[2] as f64) - (rgba2.0[2] as f64);
  let da = (rgba1.0[3] as f64) - (rgba2.0[3] as f64);

  if use_alpha {
    (dr.powi(2) + dg.powi(2) + db.powi(2) + da.powi(2)).sqrt()
  } else {
    (dr.powi(2) + dg.powi(2) + db.powi(2)).sqrt()
  }
}
