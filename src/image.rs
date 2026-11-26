use image::{Rgba, RgbaImage};
use napi::{bindgen_prelude::AsyncTask, Env, Error, Task};
use std::collections::HashMap;

use crate::colour::{color_distance, rgba_into_rgba_number};
use crate::feature_search::{find_feature, TemplatePixel};

#[napi(object)]
pub struct FeatureMatch {
  pub feature: Feature,
  pub x: u32,
  pub y: u32,
}

#[napi(object)]
#[derive(Clone)]
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
  pub fn find_rgbas(&self, rgba_number: u32, max_color_distance_percent: f64) -> AsyncTask<AsyncFindRgbas> {
    AsyncTask::new(AsyncFindRgbas::new(
      rgba_number,
      self.rgba_image.clone(),
      max_color_distance_percent,
    ))
  }

  #[napi(ts_return_type = "Promise<Array<FeatureMatch>>")]
  pub fn get_features_from_color(
    &self,
    rgba_number: u32,
    max_color_distance_percent: f64,
    max_grouping_distance: u32,
  ) -> AsyncTask<AsyncGetFeaturesFromColor> {
    AsyncTask::new(AsyncGetFeaturesFromColor::new(
      rgba_number,
      self.rgba_image.clone(),
      max_color_distance_percent,
      max_grouping_distance,
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
    Image {
      width: value.width(),
      height: value.height(),
      rgba_image: value,
    }
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
  max_color_distance_percent: f64,
}

impl AsyncFindRgbas {
  pub fn new(rgba_number: u32, rgba_image: RgbaImage, max_color_distance_percent: f64) -> Self {
    Self {
      rgba_number,
      rgba_image,
      max_color_distance_percent,
    }
  }
}

#[napi]
impl Task for AsyncFindRgbas {
  type Output = Vec<Pixel>;
  type JsValue = Vec<Pixel>;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    const MAX_COLOR_DISTANCE: f64 = 510.0; // Using alpha: sqrt(255^2 * 4)
    let actual_color_tolerance_value = MAX_COLOR_DISTANCE * self.max_color_distance_percent;
    let mut positions = Vec::new();

    for (x, y, pixel) in self.rgba_image.enumerate_pixels() {
      let pixel_rgba_u32 = rgba_into_rgba_number(pixel);
      let distance = color_distance(self.rgba_number, pixel_rgba_u32, true);

      if distance <= actual_color_tolerance_value {
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
  max_color_distance_percent: f64,
  max_grouping_distance: u32,
}

impl AsyncGetFeaturesFromColor {
  pub fn new(
    rgba_number: u32,
    rgba_image: RgbaImage,
    max_color_distance_percent: f64,
    max_grouping_distance: u32,
  ) -> Self {
    Self {
      rgba_number,
      rgba_image,
      max_color_distance_percent,
      max_grouping_distance,
    }
  }
}

#[napi]
impl Task for AsyncGetFeaturesFromColor {
  type Output = Vec<FeatureMatch>;
  type JsValue = Vec<FeatureMatch>;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let pixels = AsyncFindRgbas::new(
      self.rgba_number,
      self.rgba_image.clone(),
      self.max_color_distance_percent,
    )
    .compute()?;

    let max_dist_sq: i64 = (self.max_grouping_distance as i64) * (self.max_grouping_distance as i64);

    let mut parent: Vec<usize> = (0..pixels.len()).collect();
    fn find_set(i: usize, parent: &mut Vec<usize>) -> usize {
      if parent[i] == i {
        return i;
      }
      parent[i] = find_set(parent[i], parent);
      parent[i]
    }
    fn unite_sets(i: usize, j: usize, parent: &mut Vec<usize>) {
      let i_id = find_set(i, parent);
      let j_id = find_set(j, parent);
      if i_id != j_id {
        parent[j_id] = i_id;
      }
    }

    for i in 0..pixels.len() {
      for j in (i + 1)..pixels.len() {
        let dx = (pixels[i].x as i64) - (pixels[j].x as i64);
        let dy = (pixels[i].y as i64) - (pixels[j].y as i64);
        if dx * dx + dy * dy <= max_dist_sq {
          unite_sets(i, j, &mut parent);
        }
      }
    }

    let mut groups: HashMap<usize, Vec<Pixel>> = HashMap::new();
    for (i, pixel) in pixels.into_iter().enumerate() {
      let root = find_set(i, &mut parent);
      groups.entry(root).or_default().push(pixel);
    }

    let features = groups
      .into_values()
      .map(|mut group| {
        let min_x = group.iter().map(|p| p.x).min().unwrap();
        let min_y = group.iter().map(|p| p.y).min().unwrap();

        for pixel in &mut group {
          pixel.x -= min_x;
          pixel.y -= min_y;
        }

        let feature = Feature { pixels: group };
        FeatureMatch { feature, x: min_x, y: min_y }
      })
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
  rgba_image: RgbaImage,
}

impl AsyncFindFeatures {
  pub fn new(
    feature: Feature,
    color_tolerance_percent: f64,
    max_mismatch_percent: f64,
    rgba_image: RgbaImage,
  ) -> Self {
    Self {
      feature,
      color_tolerance_percent,
      max_mismatch_percent,
      rgba_image,
    }
  }
}

#[napi]
impl Task for AsyncFindFeatures {
  type Output = Vec<Pixel>;
  type JsValue = Vec<Pixel>;

  fn compute(&mut self) -> Result<Self::Output, Error> {
    let template: Vec<TemplatePixel> = self
      .feature
      .pixels
      .iter()
      .map(|p| TemplatePixel {
        x: p.x,
        y: p.y,
        rgba: p.rgba,
      })
      .collect();

    let found_top_lefts = find_feature(
      &self.rgba_image,
      &template,
      self.color_tolerance_percent,
      self.max_mismatch_percent,
    )
    .into_iter()
    .map(|l| Pixel {
      x: l.x,
      y: l.y,
      rgba: l.rgba,
    })
    .collect();

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