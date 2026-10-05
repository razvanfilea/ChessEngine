use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use chess_core::prelude::*;
use chess_engine::{
    board::Board,
    search::Searcher,
    time::{Instant, TimeManager},
    transposition::TranspositionTable,
};

struct EngineState {
    tt: Arc<TranspositionTable>,
    tt_size_mb: usize,
    searcher: Box<Searcher>,
}

pub struct SearchEngine {
    state: Mutex<EngineState>,
    stop_flag: Arc<AtomicBool>,
    stop_count: AtomicU64,
}

pub struct SearchResult {
    pub best_move: Move,
    pub search_time_ms: u64,
    pub uci_info: String,
}

impl SearchEngine {
    pub fn new(default_tt_size_mb: usize) -> Self {
        let tt = Arc::new(TranspositionTable::new(default_tt_size_mb));
        let stop_flag = Arc::new(AtomicBool::new(false));
        Self {
            state: Mutex::new(EngineState {
                searcher: Searcher::new(tt.clone(), stop_flag.clone()),
                tt,
                tt_size_mb: default_tt_size_mb,
            }),
            stop_flag,
            stop_count: AtomicU64::new(0),
        }
    }

    pub fn stop_count(&self) -> u64 {
        self.stop_count.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.stop_count.fetch_add(1, Ordering::SeqCst);
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    pub fn new_game(&self) {
        self.stop();
        let mut state = self.state.lock().unwrap();
        state.tt.clear();
        state.searcher.clear_histories();
    }

    pub fn search(
        &self,
        board: Board,
        position_keys: &[u64],
        depth: i32,
        max_time_ms: i64,
        hash_size_mb: i32,
        stop_count: u64,
    ) -> Option<SearchResult> {
        let mut state = self.state.lock().unwrap();
        let state = &mut *state;
        self.stop_flag.store(false, Ordering::SeqCst);
        if self.stop_count() != stop_count {
            return None;
        }

        let max_depth = if depth > 0 { depth.min(64) as u8 } else { 64 };
        let time_manager = if max_time_ms > 0 {
            let mut tm = TimeManager::from_movetime(Duration::from_millis(max_time_ms as u64));
            tm.limits.max_depth = max_depth;
            tm
        } else {
            TimeManager::from_depth(max_depth)
        };

        if hash_size_mb > 0 && hash_size_mb as usize != state.tt_size_mb {
            state.tt = Arc::new(TranspositionTable::new(hash_size_mb as usize));
            state.searcher.set_tt(state.tt.clone());
            state.tt_size_mb = hash_size_mb as usize;
        }
        state.tt.new_search();

        let start = Instant::now();
        let mut last_info = String::new();

        state
            .searcher
            .prepare_for_search(board, position_keys, time_manager);
        let best_move = state.searcher.search(|info| {
            last_info = info;
        });

        if self.stop_count() != stop_count {
            return None;
        }

        Some(SearchResult {
            best_move,
            search_time_ms: start.elapsed().as_millis() as u64,
            uci_info: last_info,
        })
    }
}
