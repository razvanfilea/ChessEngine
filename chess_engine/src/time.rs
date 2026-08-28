#[cfg(not(target_family = "wasm"))]
pub use std::time::Instant;

#[cfg(target_family = "wasm")]
#[derive(Copy, Clone, Debug, Default)]
pub struct Instant;

#[cfg(target_family = "wasm")]
impl Instant {
    #[inline(always)]
    pub fn now() -> Self {
        Self
    }

    #[inline(always)]
    pub fn elapsed(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }
}
