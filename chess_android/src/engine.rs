use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use chess_core::prelude::*;
use chess_engine::{
    board::Board,
    time::{Instant, TimeLimits, TimeManager},
    transposition::TranspositionTable,
};

pub struct SearchEngine {
    tt: Mutex<TranspositionTable>,
    tt_size_mb: AtomicUsize,
    stop_flag: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: i32,
    pub search_time_ms: u64,
    pub advanced_stats: String,
}

impl SearchEngine {
    pub fn new(default_tt_size_mb: usize) -> Self {
        Self {
            tt: Mutex::new(TranspositionTable::new(default_tt_size_mb)),
            tt_size_mb: AtomicUsize::new(default_tt_size_mb),
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    pub fn search(
        &self,
        board: Board,
        depth: i32,
        max_time_ms: i64,
        hash_size_mb: i32,
    ) -> SearchResult {
        // Reset stop flag
        self.stop_flag.store(false, Ordering::Relaxed);

        let max_depth = if depth > 0 { (depth as u8).min(64) } else { 64 };
        let time_manager = if max_time_ms > 0 {
            TimeManager {
                limits: TimeLimits {
                    max_depth,
                    max_nodes: None,
                    optimum_time: Some(Duration::from_millis(max_time_ms as u64)),
                    max_time: Some(Duration::from_millis(max_time_ms as u64)),
                    infinite: false,
                },
                start_time: Instant::now(),
            }
        } else {
            TimeManager::from_depth(max_depth)
        };

        let mut tt = self.tt.lock().unwrap();
        if hash_size_mb > 0 && hash_size_mb as usize != self.tt_size_mb.load(Ordering::Relaxed) {
            *tt = TranspositionTable::new(hash_size_mb as usize);
            self.tt_size_mb
                .store(hash_size_mb as usize, Ordering::Relaxed);
        }
        tt.new_search();

        let start = Instant::now();
        let stop_clone = self.stop_flag.clone();
        let mut last_info = String::new();

        let best_move = chess_engine::search::search(
            board.clone(),
            &[],
            time_manager,
            stop_clone,
            &tt,
            |info| {
                last_info = info;
            },
        );
        let elapsed = start.elapsed().as_millis() as u64;

        let best_move_bits = if best_move == Move::NONE || !board.legal(best_move) {
            0
        } else {
            best_move.bits() as i32
        };

        SearchResult {
            best_move: best_move_bits,
            search_time_ms: elapsed,
            advanced_stats: last_info,
        }
    }
}

