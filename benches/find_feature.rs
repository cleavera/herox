// Includes the napi-free modules directly so the benchmark doesn't link against Node.
#[path = "../src/colour.rs"]
mod colour;
#[path = "../src/feature_search.rs"]
mod feature_search;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use feature_search::{find_feature, TemplatePixel};
use image::{Rgba, RgbaImage};

// Deterministic noise so there are few accidental matches and early exit is realistic.
fn noise_image(width: u32, height: u32) -> RgbaImage {
  RgbaImage::from_fn(width, height, |x, y| {
    let h = x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663);
    let h = h ^ (h >> 13);
    Rgba([h as u8, (h >> 8) as u8, (h >> 16) as u8, 255])
  })
}

fn template_from(image: &RgbaImage, x0: u32, y0: u32, w: u32, h: u32) -> Vec<TemplatePixel> {
  (0..h)
    .flat_map(|y| (0..w).map(move |x| (x, y)))
    .map(|(x, y)| TemplatePixel {
      x,
      y,
      rgba: colour::rgba_into_rgba_number(image.get_pixel(x0 + x, y0 + y)),
    })
    .collect()
}

fn bench_find_feature(c: &mut Criterion) {
  let image = noise_image(1920, 1080);
  let template = template_from(&image, 1200, 640, 24, 24);

  let found = find_feature(&image, &template, 0.0, 0.0);
  assert!(found.iter().any(|l| (l.x, l.y) == (1200, 640)));

  let mut group = c.benchmark_group("find_feature_1920x1080_24x24");
  group.sample_size(20);

  for (name, tolerance, mismatch) in [("exact", 0.0, 0.0), ("tolerant", 0.05, 0.1)] {
    group.bench_function(name, |b| {
      b.iter(|| black_box(find_feature(&image, &template, tolerance, mismatch)))
    });
  }

  group.finish();
}

criterion_group!(benches, bench_find_feature);
criterion_main!(benches);
