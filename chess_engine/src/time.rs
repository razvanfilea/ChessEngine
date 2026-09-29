use chess_core::{Color, Move};
use std::time::Duration;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchOptions {
    pub searchmoves: Vec<Move>,
    pub wtime: Option<Duration>,
    pub btime: Option<Duration>,
    pub winc: Option<Duration>,
    pub binc: Option<Duration>,
    pub movestogo: Option<u64>,
    pub depth: Option<u64>,
    pub nodes: Option<u64>,
    pub movetime: Option<Duration>,
    pub infinite: bool,
}

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

pub const DEFAULT_MOVE_OVERHEAD_MS: u64 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeLimits {
    pub max_depth: u8,
    pub max_nodes: Option<u64>,
    pub optimum_time: Option<Duration>,
    pub max_time: Option<Duration>,
    pub infinite: bool,
}

#[derive(Clone, Debug)]
pub struct TimeManager {
    pub limits: TimeLimits,
    pub start_time: Instant,
}

impl TimeManager {
    pub fn from_depth(depth: u8) -> Self {
        Self {
            limits: TimeLimits {
                max_depth: depth.clamp(1, 64),
                max_nodes: None,
                optimum_time: None,
                max_time: None,
                infinite: false,
            },
            start_time: Instant::now(),
        }
    }

    pub fn from_movetime(movetime: Duration) -> Self {
        Self::from_movetime_with_overhead(movetime, DEFAULT_MOVE_OVERHEAD_MS)
    }

    pub fn from_movetime_with_overhead(movetime: Duration, move_overhead_ms: u64) -> Self {
        let safe = movetime
            .saturating_sub(Duration::from_millis(move_overhead_ms))
            .max(Duration::from_millis(1));
        Self {
            limits: TimeLimits {
                max_depth: 64,
                max_nodes: None,
                optimum_time: Some(safe),
                max_time: Some(safe),
                infinite: false,
            },
            start_time: Instant::now(),
        }
    }

    pub fn from_nodes(nodes: u64) -> Self {
        Self {
            limits: TimeLimits {
                max_depth: 64,
                max_nodes: Some(nodes.max(1)),
                optimum_time: None,
                max_time: None,
                infinite: false,
            },
            start_time: Instant::now(),
        }
    }

    pub fn infinite() -> Self {
        Self {
            limits: TimeLimits {
                max_depth: 64,
                max_nodes: None,
                optimum_time: None,
                max_time: None,
                infinite: true,
            },
            start_time: Instant::now(),
        }
    }

    pub fn from_options(opts: &SearchOptions, to_play: Color, move_overhead_ms: u64) -> Self {
        let start_time = Instant::now();
        let max_depth = opts.depth.map_or(64, |d| d.clamp(1, 64) as u8);
        let max_nodes = opts.nodes.map(|n| n.max(1));
        let infinite = opts.infinite;

        if let Some(movetime) = opts.movetime {
            let safe = movetime
                .saturating_sub(Duration::from_millis(move_overhead_ms))
                .max(Duration::from_millis(1));
            return Self {
                limits: TimeLimits {
                    max_depth,
                    max_nodes,
                    optimum_time: Some(safe),
                    max_time: Some(safe),
                    infinite,
                },
                start_time,
            };
        }

        let (time, inc) = match to_play {
            Color::White => (opts.wtime, opts.winc.unwrap_or(Duration::ZERO)),
            Color::Black => (opts.btime, opts.binc.unwrap_or(Duration::ZERO)),
        };

        let (optimum_time, max_time) = if let Some(remaining) = time {
            let moves_to_go = opts.movestogo.unwrap_or(40).clamp(1, 50);
            let time_ms = remaining.as_millis() as u64;
            let inc_ms = inc.as_millis() as u64;

            const MAX_TIME_MULTIPLIER: u64 = 5;
            const MAX_TIME_DIVISOR: u64 = 1;

            let usable_time = time_ms.saturating_sub(move_overhead_ms);
            let mut opt_ms = (usable_time / moves_to_go) + (inc_ms * 7 / 10);
            opt_ms = opt_ms.clamp(1, usable_time.max(1));

            let max_ms = (opt_ms * MAX_TIME_MULTIPLIER / MAX_TIME_DIVISOR)
                .clamp(opt_ms, usable_time.max(opt_ms));

            (
                Some(Duration::from_millis(opt_ms)),
                Some(Duration::from_millis(max_ms)),
            )
        } else {
            (None, None)
        };

        Self {
            limits: TimeLimits {
                max_depth,
                max_nodes,
                optimum_time,
                max_time,
                infinite,
            },
            start_time,
        }
    }

    #[inline(always)]
    pub fn is_hard_limit_exceeded(&self, nodes_searched: u64) -> bool {
        if self.limits.infinite {
            return false;
        }
        if let Some(max_nodes) = self.limits.max_nodes
            && nodes_searched >= max_nodes
        {
            return true;
        }
        if let Some(max_time) = self.limits.max_time
            && self.start_time.elapsed() >= max_time
        {
            return true;
        }
        false
    }

    pub fn should_stop_after_depth(&self, depth: u8, best_move_stable: bool) -> bool {
        if self.limits.infinite {
            return false;
        }
        if depth >= self.limits.max_depth {
            return true;
        }
        if let Some(optimum) = self.limits.optimum_time {
            let elapsed = self.start_time.elapsed();
            let mut target = optimum;

            if !best_move_stable {
                target = target.saturating_add(target * 3 / 10);
            }

            if elapsed >= target {
                return true;
            }
        }
        false
    }
}
