#[cfg(not(target_family = "wasm"))]
pub use std::time::Instant;

#[cfg(target_family = "wasm")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn performance_now() -> f64;
}

#[cfg(target_family = "wasm")]
#[derive(Copy, Clone, Debug)]
pub struct Instant(f64);

#[cfg(target_family = "wasm")]
impl Instant {
    #[inline(always)]
    pub fn now() -> Self {
        Self(unsafe { performance_now() })
    }

    #[inline(always)]
    pub fn elapsed(&self) -> std::time::Duration {
        let now = unsafe { performance_now() };
        let ms = (now - self.0).max(0.0);
        std::time::Duration::from_secs_f64(ms / 1000.0)
    }
}
