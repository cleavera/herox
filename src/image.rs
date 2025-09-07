use image::{Rgba, RgbaImage};
use napi::{bindgen_prelude::AsyncTask, Env, Error, Task};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static IMAGE_COUNT: AtomicUsize = AtomicUsize::new(0);

#[napi(object)]
pub struct Pixel {
  pub x: u32,
  pub y: u32,
  pub rgba: u32,
}

#[napi(object)]
pub struct Feature {
  pub pixels: Vec<Pixel>,
}

#[napi(object)]
pub struct ColourFrequency {
  pub rgba: u32,
  pub count: u32,
}

#[napi]
#[derive(Debug, Clone)]
pub struct Image {
  rgba_image: RgbaImage,
  #[napi(readonly)]
  pub width: u32,
  #[napi(readonly)]
  pub height: u32,
}

#[napi]
impl Image {
  #[napi(ts_return_type = "Promise<number>")]
  pub fn get_pixel_rgba(&self, x: u32, y: u32) -> AsyncTask<AsyncGetPixelRgba> {
    AsyncTask::new(AsyncGetPixelRgba::new(x, y, self.rgba_image.clone()))
  }

  #[napi]
  pub fn get_pixel_rgba_sync(&self, x: u32, y: u32) -> Result<u32, Error> {
    AsyncGetPixelRgba::new(x, y, self.rgba_image.clone()).compute()
  }

  #[napi(ts_return_type = "Promise<Array<Pixel>>")]
  pub fn find_rgbas(&self, rgba_number: u32) -> AsyncTask<AsyncFindRgbas> {
    AsyncTask::new(AsyncFindRgbas::new(
      rgba_number,
      self.rgba_image.clone(),
    ))
  }

  #[napi(ts_return_type = "Promise<Array<Feature>>")]
  pub fn get_features_from_color(
    &self,
    rgba_number: u32,
  ) -> AsyncTask<AsyncGetFeaturesFromColor> {
    AsyncTask::new(AsyncGetFeaturesFromColor::new(
      rgba_number,
      self.rgba_image.clone(),
    ))
  }

  #[napi(ts_return_type = "Promise<Array<Pixel>>")]
  pub fn find_feature(
    &self,
    feature: Feature,
    max_color_distance_percent: f64,
    max_pixel_difference_percent: f64,
  ) -> AsyncTask<AsyncFindFeatures> {
    AsyncTask::new(AsyncFindFeatures::new(
      feature,
      max_color_distance_percent,
      max_pixel_difference_percent,
      self.width,
      self.height,
      self.rgba_image.clone(),
    ))
  }

  #[napi(ts_return_type = "Promise<number>")]
  pub fn check_feature(
    &self,
    x: u32,
    y: u32,
    feature: Feature,
    max_color_distance_percent: f64,
  ) -> AsyncTask<AsyncCheckFeature> {
    AsyncTask::new(AsyncCheckFeature::new(
      x,
      y,
      feature,
      self.rgba_image.clone(),
      max_color_distance_percent,
    ))
  }

  #[napi(ts_return_type = "Promise<Feature>")]
  pub fn get_feature(
    &self,
    start_x: u32,
    start_y: u32,
    end_x: u32,
    end_y: u32,
  ) -> AsyncTask<AsyncGetFeature> {
    AsyncTask::new(AsyncGetFeature::new(
      start_x,
      start_y,
      end_x,
      end_y,
      self.rgba_image.clone(),
    ))
  }

  #[napi(ts_return_type = "Promise<Array<ColourFrequency>>")]
  pub fn get_colour_frequencies(
    &self,
    start_x: u32,
    start_y: u32,
    end_x: u32,
    end_y: u32,
  ) -> AsyncTask<AsyncGetColourFrequencies> {
    AsyncTask::new(AsyncGetColourFrequencies::new(
      start_x,
      start_y,
      end_x,
      end_y,
      self.rgba_image.clone(),
    ))
  }
}

impl From<RgbaImage> for Image {
  fn from(value: RgbaImage) -> Self {
      IMAGE_COUNT.fetch_add(1, Ordering::SeqCst);

      if IMAGE_COUNT.load(Ordering::SeqCst) > 20 {
        panic!("Too many images");
      }

    Image {
      width: value.width(),
      height: value.height(),
      rgba_image: value,
    }
  }
}

impl Drop for Image {
    fn drop(&mut self) {
        IMAGE_COUNT.fetch_sub(1, Ordering::SeqCst);
    }
}

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

fn color_distance(color1_u32: u32, color2_u32: u32, use_alpha: bool) -> f64 {
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

pub struct AsyncGetPixelRgba {
  x: u32,
  y: u32,
  rgba_image: RgbaImage,
}

impl AsyncGetPixelRgba {
  pub fn new(x: u32, y: u32, rgba_image: RgbaImage) -> Self {
    Self { x, y, rgba_image }
  }
}

#[napi]
impl Task for AsyncGetPixelRgba {
  type Output = u32;
  type JsValue = u32;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let rgba_pixel: &Rgba<u8> = self
      .rgba_image
      .get_pixel_checked(self.x, self.y)
      .ok_or(Error::from_reason("Pixel out of bounds"))?;

    Ok(rgba_into_rgba_number(rgba_pixel))
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue, Error> {
    Ok(output)
  }
}

pub struct AsyncFindRgbas {
  rgba_number: u32,
  rgba_image: RgbaImage,
}

impl AsyncFindRgbas {
  pub fn new(rgba_number: u32, rgba_image: RgbaImage) -> Self {
    Self {
      rgba_number,
      rgba_image,
    }
  }
}

#[napi]
impl Task for AsyncFindRgbas {
  type Output = Vec<Pixel>;
  type JsValue = Vec<Pixel>;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let rgba = rgba_number_into_rgba(self.rgba_number);
    let mut positions = Vec::new();

    for (x, y, pixel) in self.rgba_image.enumerate_pixels() {
      if *pixel == rgba {
        positions.push(Pixel {
          x,
          y,
          rgba: rgba_into_rgba_number(pixel),
        });
      }
    }

    Ok(positions)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue, Error> {
    Ok(output)
  }
}

pub struct AsyncGetFeaturesFromColor {
  rgba_number: u32,
  rgba_image: RgbaImage,
}

impl AsyncGetFeaturesFromColor {
  pub fn new(rgba_number: u32, rgba_image: RgbaImage) -> Self {
    Self {
      rgba_number,
      rgba_image,
    }
  }
}

#[napi]
impl Task for AsyncGetFeaturesFromColor {
  type Output = Vec<Feature>;
  type JsValue = Vec<Feature>;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let mut find_task = AsyncFindRgbas::new(self.rgba_number, self.rgba_image.clone());
    let mut pixels = find_task.compute()?;

    pixels.sort_by_key(|p| (p.x, p.y));

    const MAX_DISTANCE: u32 = 5;
    const MAX_DIST_SQ: i64 = (MAX_DISTANCE as i64) * (MAX_DISTANCE as i64);

    let mut groups: Vec<Vec<Pixel>> = Vec::new();

    for pixel in &pixels {
      let mut found_group_for_pixel = false;
      for group in &mut groups {
        if group.iter().any(|gp| {
          let dx = (gp.x as i64) - (pixel.x as i64);
          let dy = (gp.y as i64) - (pixel.y as i64);
          dx * dx + dy * dy <= MAX_DIST_SQ
        }) {
          group.push(pixel.clone());
          found_group_for_pixel = true;
          break;
        }
      }

      if !found_group_for_pixel {
        groups.push(vec![pixel.clone()]);
      }
    }

    let features = groups
      .into_iter()
      .map(|g| Feature { pixels: g })
      .collect();

    Ok(features)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue, Error> {
    Ok(output)
  }
}

pub struct AsyncFindFeatures {
  feature: Feature,
  color_tolerance_percent: f64,
  max_mismatch_percent: f64,
  width: u32,
  height: u32,
  rgba_image: RgbaImage,
}

impl AsyncFindFeatures {
  pub fn new(
    feature: Feature,
    color_tolerance_percent: f64,
    max_mismatch_percent: f64,
    width: u32,
    height: u32,
    rgba_image: RgbaImage,
  ) -> Self {
    Self {
      feature,
      color_tolerance_percent,
      max_mismatch_percent,
      width,
      height,
      rgba_image,
    }
  }
}

#[napi]
impl Task for AsyncFindFeatures {
  type Output = Vec<Pixel>;
  type JsValue = Vec<Pixel>;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let mut found_top_lefts = Vec::new();

    if self.feature.pixels.is_empty() {
      return Ok(found_top_lefts);
    }

    let min_feat_x = self.feature.pixels.iter().map(|p| p.x).min().unwrap_or(0);
    let min_feat_y = self.feature.pixels.iter().map(|p| p.y).min().unwrap_or(0);
    let max_feat_x = self.feature.pixels.iter().map(|p| p.x).max().unwrap_or(0);
    let max_feat_y = self.feature.pixels.iter().map(|p| p.y).max().unwrap_or(0);

    let feature_width = max_feat_x - min_feat_x + 1;
    let feature_height = max_feat_y - min_feat_y + 1;

    if feature_width > self.width || feature_height > self.height {
      return Ok(found_top_lefts);
    }

    let max_color_distance: f64 = if true {
      510.0 // sqrt(255*255 * 4)
    } else {
      441.67 // sqrt(255*255 * 3)
    };
    let actual_color_tolerance_value = max_color_distance * self.color_tolerance_percent;

    let total_feature_pixels = self.feature.pixels.len() as f64;
    let max_mismatches_count = (total_feature_pixels * self.max_mismatch_percent).round() as u32;

    let use_alpha_for_comparison = true;

    for start_y in 0..=(self.height - feature_height) {
      for start_x in 0..=(self.width - feature_width) {
        let mut current_mismatches = 0;

        for feature_pixel in &self.feature.pixels {
          let current_image_x = start_x + (feature_pixel.x - min_feat_x);
          let current_image_y = start_y + (feature_pixel.y - min_feat_y);

          let image_rgba_raw = self
            .rgba_image
            .get_pixel_checked(current_image_x, current_image_y);

          match image_rgba_raw {
            Some(img_pixel_rgba) => {
              let img_pixel_rgba_u32 = rgba_into_rgba_number(img_pixel_rgba);
              let distance = color_distance(
                feature_pixel.rgba,
                img_pixel_rgba_u32,
                use_alpha_for_comparison,
              );

              if distance > actual_color_tolerance_value {
                current_mismatches += 1;
                if current_mismatches > max_mismatches_count {
                  break;
                }
              }
            }
            None => {
              current_mismatches += 1;
              if current_mismatches > max_mismatches_count {
                break;
              }
            }
          }
        }

        if current_mismatches <= max_mismatches_count {
          let top_left_pixel_rgba_raw = self.rgba_image.get_pixel(start_x, start_y);
          let top_left_rgba_u32 = rgba_into_rgba_number(top_left_pixel_rgba_raw);

          found_top_lefts.push(Pixel {
            x: start_x,
            y: start_y,
            rgba: top_left_rgba_u32,
          });
        }
      }
    }

    Ok(found_top_lefts)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue, Error> {
    Ok(output)
  }
}

pub struct AsyncCheckFeature {
  x: u32,
  y: u32,
  feature: Feature,
  color_tolerance_percent: f64,
  width: u32,
  height: u32,
  rgba_image: RgbaImage,
}

impl AsyncCheckFeature {
  pub fn new(
    x: u32,
    y: u32,
    feature: Feature,
    rgba_image: RgbaImage,
    color_tolerance_percent: f64,
  ) -> Self {
    Self {
      x,
      y,
      feature,
      color_tolerance_percent,
      width: rgba_image.width(),
      height: rgba_image.height(),
      rgba_image,
    }
  }
}

#[napi]
impl Task for AsyncCheckFeature {
  type Output = f64;
  type JsValue = f64;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    if self.feature.pixels.is_empty() {
      return Err(Error::from_reason("This feature has no pixels"));
    }

    let min_feat_x = self.feature.pixels.iter().map(|p| p.x).min().unwrap_or(0);
    let min_feat_y = self.feature.pixels.iter().map(|p| p.y).min().unwrap_or(0);
    let max_feat_x = self.feature.pixels.iter().map(|p| p.x).max().unwrap_or(0);
    let max_feat_y = self.feature.pixels.iter().map(|p| p.y).max().unwrap_or(0);

    let feature_width = max_feat_x - min_feat_x + 1;
    let feature_height = max_feat_y - min_feat_y + 1;

    if self.x + feature_width > self.width || self.y + feature_height > self.height {
      return Err(Error::from_reason(
        "Feature, when placed at the given top_left point, extends beyond image boundaries.",
      ));
    }

    const MAX_COLOR_DISTANCE: f64 = 510.0;
    let actual_color_tolerance_value = MAX_COLOR_DISTANCE * self.color_tolerance_percent;
    let use_alpha_for_comparison = true;

    let mut matching_pixels_count = 0;
    let total_pixels_to_check = self.feature.pixels.len();

    for feature_pixel in &self.feature.pixels {
      let current_image_x = self.x + (feature_pixel.x - min_feat_x);
      let current_image_y = self.y + (feature_pixel.y - min_feat_y);

      if let Some(img_pixel_rgba) = self
        .rgba_image
        .get_pixel_checked(current_image_x, current_image_y)
      {
        let img_pixel_rgba_u32 = rgba_into_rgba_number(img_pixel_rgba);
        let distance = color_distance(
          feature_pixel.rgba,
          img_pixel_rgba_u32,
          use_alpha_for_comparison,
        );

        if distance <= actual_color_tolerance_value {
          matching_pixels_count += 1;
        }
      }
    }

    let percentage_match = matching_pixels_count as f64 / total_pixels_to_check as f64;

    Ok(percentage_match)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue, Error> {
    Ok(output)
  }
}

pub struct AsyncGetFeature {
  start_x: u32,
  start_y: u32,
  end_x: u32,
  end_y: u32,
  width: u32,
  height: u32,
  rgba_image: RgbaImage,
}

impl AsyncGetFeature {
  pub fn new(start_x: u32, start_y: u32, end_x: u32, end_y: u32, rgba_image: RgbaImage) -> Self {
    Self {
      start_x,
      start_y,
      end_x,
      end_y,
      width: rgba_image.width(),
      height: rgba_image.height(),
      rgba_image,
    }
  }
}

#[napi]
impl Task for AsyncGetFeature {
  type Output = Feature;
  type JsValue = Feature;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let min_x = self.start_x.min(self.end_x);
    let max_x = self.start_x.max(self.end_x);
    let min_y = self.start_y.min(self.end_y);
    let max_y = self.start_y.max(self.end_y);

    if min_x >= self.width || min_y >= self.height {
      return Err(Error::from_reason(
        "Start point is outside image boundaries.",
      ));
    }

    if max_x >= self.width || max_y >= self.height {
      return Err(Error::from_reason("End point is outside image boundaries."));
    }

    let mut pixels_in_region: Vec<Pixel> = Vec::new();

    for y in min_y..=max_y {
      for x in min_x..=max_x {
        let rgba_raw = self.rgba_image.get_pixel(x, y);
        let rgba_u32 = rgba_into_rgba_number(rgba_raw);

        pixels_in_region.push(Pixel {
          x: x - min_x, // Relative x
          y: y - min_y, // Relative y
          rgba: rgba_u32,
        });
      }
    }

    Ok(Feature {
      pixels: pixels_in_region,
    })
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue, Error> {
    Ok(output)
  }
}

pub struct AsyncGetColourFrequencies {
  start_x: u32,
  start_y: u32,
  end_x: u32,
  end_y: u32,
  rgba_image: RgbaImage,
}

impl AsyncGetColourFrequencies {
  pub fn new(start_x: u32, start_y: u32, end_x: u32, end_y: u32, rgba_image: RgbaImage) -> Self {
    Self {
      start_x,
      start_y,
      end_x,
      end_y,
      rgba_image,
    }
  }
}

#[napi]
impl Task for AsyncGetColourFrequencies {
  type Output = Vec<ColourFrequency>;
  type JsValue = Vec<ColourFrequency>;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let min_x = self.start_x.min(self.end_x);
    let max_x = self.start_x.max(self.end_x);
    let min_y = self.start_y.min(self.end_y);
    let max_y = self.start_y.max(self.end_y);

    let width = self.rgba_image.width();
    let height = self.rgba_image.height();

    if min_x >= width || min_y >= height || max_x >= width || max_y >= height {
      return Err(Error::from_reason(
        "Coordinates are outside image boundaries.",
      ));
    }

    let mut colour_counts: HashMap<u32, u32> = HashMap::new();

    for y in min_y..=max_y {
      for x in min_x..=max_x {
        let rgba_raw = self.rgba_image.get_pixel(x, y);
        let rgba_u32 = rgba_into_rgba_number(rgba_raw);
        *colour_counts.entry(rgba_u32).or_insert(0) += 1;
      }
    }

    let frequencies = colour_counts
      .into_iter()
      .map(|(rgba, count)| ColourFrequency { rgba, count })
      .collect();

    Ok(frequencies)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue, Error> {
    Ok(output)
  }
}
