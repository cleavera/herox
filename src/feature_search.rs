use image::RgbaImage;
use rayon::prelude::*;

use crate::colour::{rgba_into_rgba_number, rgba_number_into_rgba};

pub struct TemplatePixel {
  pub x: u32,
  pub y: u32,
  pub rgba: u32,
}

pub struct FeatureLocation {
  pub x: u32,
  pub y: u32,
  pub rgba: u32,
}

pub fn find_feature(
  rgba_image: &RgbaImage,
  feature_pixels: &[TemplatePixel],
  color_tolerance_percent: f64,
  max_mismatch_percent: f64,
) -> Vec<FeatureLocation> {
  let width = rgba_image.width();
  let height = rgba_image.height();

  if feature_pixels.is_empty() {
    return Vec::new();
  }

  let min_feat_x = feature_pixels.iter().map(|p| p.x).min().unwrap_or(0);
  let min_feat_y = feature_pixels.iter().map(|p| p.y).min().unwrap_or(0);
  let max_feat_x = feature_pixels.iter().map(|p| p.x).max().unwrap_or(0);
  let max_feat_y = feature_pixels.iter().map(|p| p.y).max().unwrap_or(0);

  let feature_width = max_feat_x - min_feat_x + 1;
  let feature_height = max_feat_y - min_feat_y + 1;

  if feature_width > width || feature_height > height {
    return Vec::new();
  }

  let max_color_distance: f64 = 510.0; // sqrt(255*255 * 4), alpha included
  let actual_color_tolerance_value = max_color_distance * color_tolerance_percent;
  let tolerance_squared = actual_color_tolerance_value * actual_color_tolerance_value;

  let total_feature_pixels = feature_pixels.len() as f64;
  let max_mismatches_count = (total_feature_pixels * max_mismatch_percent).round() as u32;

  // Pre-unpack the feature so the inner loop is plain integer maths on the raw buffer.
  let stride = width as usize;
  let template: Vec<(usize, [i32; 4])> = feature_pixels
    .iter()
    .map(|p| {
      let rgba = rgba_number_into_rgba(p.rgba).0;
      let offset = (p.y - min_feat_y) as usize * stride + (p.x - min_feat_x) as usize;
      (
        offset,
        [rgba[0] as i32, rgba[1] as i32, rgba[2] as i32, rgba[3] as i32],
      )
    })
    .collect();

  let raw = rgba_image.as_raw();
  let max_x = width - feature_width;

  (0..=(height - feature_height))
    .into_par_iter()
    .flat_map_iter(|start_y| {
      let template = &template;

      (0..=max_x).filter_map(move |start_x| {
        let base = start_y as usize * stride + start_x as usize;
        let mut current_mismatches = 0;

        for (offset, expected) in template {
          let i = (base + offset) * 4;
          let dr = expected[0] - raw[i] as i32;
          let dg = expected[1] - raw[i + 1] as i32;
          let db = expected[2] - raw[i + 2] as i32;
          let da = expected[3] - raw[i + 3] as i32;
          let distance_squared = (dr * dr + dg * dg + db * db + da * da) as f64;

          if distance_squared > tolerance_squared {
            current_mismatches += 1;
            if current_mismatches > max_mismatches_count {
              return None;
            }
          }
        }

        Some(FeatureLocation {
          x: start_x,
          y: start_y,
          rgba: rgba_into_rgba_number(rgba_image.get_pixel(start_x, start_y)),
        })
      })
    })
    .collect()
}
