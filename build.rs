extern crate napi_build;

fn main() {
  napi_build::setup();

  // Benches are plain executables, so the N-API symbols Node normally provides are
  // unresolved at link time. The benchmarks never call into them.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
    println!("cargo:rustc-link-arg-benches=-Wl,--unresolved-symbols=ignore-all,-z,lazy,-z,norelro");
  }
}
