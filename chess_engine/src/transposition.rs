use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use chess_core::Move;

use crate::eval::{EVAL_NONE, MATE_THRESHOLD};

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TTFlag {
    #[default]
    Exact = 0,
    LowerBound = 1,
    UpperBound = 2,
}

impl TTFlag {
    #[inline(always)]
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0b0011 {
            0 => Self::Exact,
            1 => Self::LowerBound,
            _ => Self::UpperBound,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TTEntry {
    pub mov: Move,
    pub eval: i16,
    pub value: i16,
    pub depth: u8,
    flag_age: u8,
}

impl Default for TTEntry {
    #[inline(always)]
    fn default() -> Self {
        Self {
            mov: Move::NONE,
            eval: EVAL_NONE,
            value: 0,
            depth: 0,
            flag_age: 0,
        }
    }
}

impl TTEntry {
    #[inline(always)]
    pub const fn new(mov: Move, value: i16, eval: i16, depth: u8, flag: TTFlag) -> Self {
        Self {
            mov,
            eval,
            value,
            depth,
            flag_age: flag as u8,
        }
    }

    #[inline(always)]
    pub const fn new_with_age(
        mov: Move,
        value: i16,
        eval: i16,
        depth: u8,
        flag: TTFlag,
        age: u8,
    ) -> Self {
        Self {
            mov,
            eval,
            value,
            depth,
            flag_age: ((age & 0x3F) << 2) | (flag as u8),
        }
    }

    #[inline(always)]
    pub fn set_age(&mut self, age: u8) {
        self.flag_age = (self.flag_age & 0b0011) | ((age & 0x3F) << 2);
    }

    #[inline(always)]
    pub fn flag(&self) -> TTFlag {
        TTFlag::from_bits(self.flag_age)
    }

    #[inline(always)]
    pub fn age(&self) -> u8 {
        self.flag_age >> 2
    }

    #[inline(always)]
    pub fn flag_and_age(&self) -> (TTFlag, u8) {
        (self.flag(), self.age())
    }

    #[inline(always)]
    pub fn score_to_tt(score: i16, ply: u16) -> i16 {
        if score > MATE_THRESHOLD {
            score + ply as i16
        } else if score < -MATE_THRESHOLD {
            score - ply as i16
        } else {
            score
        }
    }

    #[inline(always)]
    pub fn score_from_tt(score: i16, ply: u16) -> i16 {
        if score > MATE_THRESHOLD {
            score - ply as i16
        } else if score < -MATE_THRESHOLD {
            score + ply as i16
        } else {
            score
        }
    }

    #[inline(always)]
    fn to_bits(self) -> u64 {
        (self.mov.bits() as u64)
            | (((self.eval as u16) as u64) << 16)
            | (((self.value as u16) as u64) << 32)
            | ((self.depth as u64) << 48)
            | ((self.flag_age as u64) << 56)
    }

    #[inline(always)]
    fn from_bits(bits: u64) -> Self {
        Self {
            mov: unsafe { Move::from_bits_unchecked(bits as u16) },
            eval: (bits >> 16) as u16 as i16,
            value: (bits >> 32) as u16 as i16,
            depth: (bits >> 48) as u8,
            flag_age: (bits >> 56) as u8,
        }
    }
}

/// Atomic 16-byte slot implementing lockless XOR verification.
#[derive(Default)]
struct AtomicTTEntry {
    data: AtomicU64,
    checksum: AtomicU64,
}

impl AtomicTTEntry {
    #[inline(always)]
    fn clear(&self) {
        self.data.store(0, Relaxed);
        self.checksum.store(0, Relaxed);
    }

    #[inline(always)]
    fn load(&self) -> (TTEntry, u64) {
        let raw_data = self.data.load(Relaxed);
        let checksum = self.checksum.load(Relaxed);
        let entry_key = raw_data ^ checksum;
        (TTEntry::from_bits(raw_data), entry_key)
    }

    #[inline(always)]
    fn save(&self, entry: TTEntry, hash: u64) {
        let raw_data = entry.to_bits();
        self.data.store(raw_data, Relaxed);
        self.checksum.store(raw_data ^ hash, Relaxed);
    }
}

const BUCKET_SIZE: usize = 4;

/// Cache-line aligned (64 bytes) bucket
#[repr(align(64))]
#[derive(Default)]
struct TTBucket([AtomicTTEntry; BUCKET_SIZE]);

pub struct TranspositionTable {
    buckets: Box<[TTBucket]>,
    /// Current search generation (0..=63, matching `flag_age >> 2`).
    age: u8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let bucket_count = ((size_mb * 1024 * 1024) / std::mem::size_of::<TTBucket>()).max(1);
        let buckets = (0..bucket_count).map(|_| TTBucket::default()).collect();
        Self { buckets, age: 0 }
    }

    #[inline(always)]
    pub fn new_search(&mut self) {
        self.age = (self.age + 1) & 0x3F;
    }

    pub fn clear(&self) {
        for bucket in &self.buckets {
            for slot in &bucket.0 {
                slot.clear();
            }
        }
    }

    #[inline(always)]
    fn bucket(&self, hash: u64) -> &[AtomicTTEntry; BUCKET_SIZE] {
        // Lemire fastrange mapping across high bits of hash
        let idx = ((hash as u128 * self.buckets.len() as u128) >> 64) as usize;
        unsafe { &self.buckets.get_unchecked(idx).0 }
    }

    pub fn probe(&self, hash: u64, ply: u16) -> Option<TTEntry> {
        for slot in self.bucket(hash) {
            let (mut entry, entry_key) = slot.load();
            if entry_key == hash {
                entry.value = TTEntry::score_from_tt(entry.value, ply);
                return Some(entry);
            }
        }
        None
    }

    pub fn store(&self, hash: u64, mut entry: TTEntry, ply: u16) {
        entry.value = TTEntry::score_to_tt(entry.value, ply);
        entry.set_age(self.age);

        let bucket = self.bucket(hash);
        let (victim_idx, victim_entry, victim_hash) = self.select_victim(bucket, hash);
        let victim_slot = &bucket[victim_idx];
        let same_pos = victim_hash == hash;

        // Preserve previous move & eval if not provided for the same position
        if entry.mov.is_none() && same_pos {
            entry.mov = victim_entry.mov;
        }
        if entry.eval == EVAL_NONE && same_pos {
            entry.eval = victim_entry.eval;
        }

        // Avoid overwriting a deep entry from the current generation with a shallow bound
        if same_pos {
            let is_current_search = victim_entry.age() == self.age;
            if is_current_search
                && entry.flag() != TTFlag::Exact
                && (entry.depth as i16) + 3 < (victim_entry.depth as i16)
            {
                // If we found a move where none was recorded, update only the move
                if victim_entry.mov.is_none() && !entry.mov.is_none() {
                    let mut updated = victim_entry;
                    updated.mov = entry.mov;
                    victim_slot.save(updated, hash);
                }
                return;
            }
        }

        victim_slot.save(entry, hash);
    }

    /// Selects an existing entry for the same position, or the lowest quality entry.
    #[inline(always)]
    fn select_victim(
        &self,
        bucket: &[AtomicTTEntry; BUCKET_SIZE],
        hash: u64,
    ) -> (usize, TTEntry, u64) {
        let mut best_idx = 0;
        let mut best_entry = TTEntry::default();
        let mut best_hash = 0;
        let mut min_quality = i32::MAX;

        for (i, slot) in bucket.iter().enumerate() {
            let (entry, entry_hash) = slot.load();
            if entry_hash == hash {
                return (i, entry, entry_hash);
            }
            let q = self.quality(&entry);
            if q < min_quality {
                min_quality = q;
                best_idx = i;
                best_entry = entry;
                best_hash = entry_hash;
            }
        }

        (best_idx, best_entry, best_hash)
    }

    /// Lower = more replaceable. Rewards depth, penalizes stale search generations.
    #[inline(always)]
    fn quality(&self, entry: &TTEntry) -> i32 {
        let age_diff = (self.age.wrapping_sub(entry.age()) & 0x3F) as i32;
        entry.depth as i32 - age_diff * 4
    }

    /// Returns the approximate hash table fill rate in per-mille (0..1000).
    pub fn hashfull(&self) -> usize {
        let sample_size = self.buckets.len().min(1000);
        let mut used = 0;
        for bucket in &self.buckets[..sample_size] {
            for slot in &bucket.0 {
                let (entry, _) = slot.load();
                if entry.depth > 0 && entry.age() == self.age {
                    used += 1;
                }
            }
        }
        (used * 1000) / (sample_size * BUCKET_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::{MoveFlags, Sq};

    #[test]
    fn test_store_and_probe() {
        let tt = TranspositionTable::new(1);
        let mov = Move::new(Sq::E2, Sq::E4, MoveFlags::DoublePawn);
        let entry = TTEntry::new(mov, 150, 140, 6, TTFlag::Exact);
        tt.store(0x1234_5678_9ABC_DEF0, entry, 2);

        let probed = tt
            .probe(0x1234_5678_9ABC_DEF0, 2)
            .expect("Entry must be found");
        assert_eq!(probed.value, 150);
        assert_eq!(probed.eval, 140);
        assert_eq!(probed.depth, 6);
        assert_eq!(probed.mov, mov);
        assert_eq!(probed.flag(), TTFlag::Exact);

        // Probing with different hash returns None
        assert!(tt.probe(0x9999_8888, 2).is_none());
    }

    #[test]
    fn test_mate_score_adjustment() {
        let tt = TranspositionTable::new(1);
        let mate_score = 29_500; // Mate in 500 at ply 4
        let entry = TTEntry::new(Move::NONE, mate_score, EVAL_NONE, 8, TTFlag::Exact);
        tt.store(0xCAFE_BABE, entry, 4);

        // At ply 4 probe returns same mate score
        let entry_ply4 = tt.probe(0xCAFE_BABE, 4).unwrap();
        assert_eq!(entry_ply4.value, mate_score);

        // At ply 2 probe adjusts to distance from ply 2
        let entry_ply2 = tt.probe(0xCAFE_BABE, 2).unwrap();
        assert_eq!(entry_ply2.value, mate_score + 2);
    }

    #[test]
    fn test_move_and_eval_preservation() {
        let tt = TranspositionTable::new(1);
        let hash = 0xABCD_EF01_2345;
        let mov = Move::new(Sq::E2, Sq::E4, MoveFlags::DoublePawn);

        // 1. First store with a valid move and static eval
        let entry1 = TTEntry::new(mov, 100, 35, 4, TTFlag::Exact);
        tt.store(hash, entry1, 0);
        let entry = tt.probe(hash, 0).unwrap();
        assert_eq!(entry.mov, mov);
        assert_eq!(entry.eval, 35);

        // 2. Second store with Move::NONE and EVAL_NONE (e.g. on UpperBound fail-low at depth 5)
        let entry2 = TTEntry::new(Move::NONE, 80, EVAL_NONE, 5, TTFlag::UpperBound);
        tt.store(hash, entry2, 0);
        let entry = tt.probe(hash, 0).unwrap();
        // Move and eval must be preserved!
        assert_eq!(entry.mov, mov);
        assert_eq!(entry.eval, 35);
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.flag(), TTFlag::UpperBound);
    }

    #[test]
    fn test_hashfull_and_aging() {
        let mut tt = TranspositionTable::new(1);
        assert_eq!(tt.hashfull(), 0);

        let mov = Move::new(Sq::E2, Sq::E4, MoveFlags::DoublePawn);
        for i in 0..4000 {
            let entry = TTEntry::new(mov, 100, 50, 4, TTFlag::Exact);
            tt.store((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15), entry, 0);
        }
        assert!(tt.hashfull() > 0);

        // Advancing search generation ages entries
        tt.new_search();
        assert_eq!(tt.hashfull(), 0);
    }

    #[test]
    fn test_entry_packing_roundtrip() {
        let mov = Move::new(Sq::A1, Sq::H8, MoveFlags::Capture);
        let entry = TTEntry::new_with_age(mov, -1500, 320, 12, TTFlag::LowerBound, 42);
        let bits = entry.to_bits();
        let unpacked = TTEntry::from_bits(bits);

        assert_eq!(unpacked.mov, mov);
        assert_eq!(unpacked.value, -1500);
        assert_eq!(unpacked.eval, 320);
        assert_eq!(unpacked.depth, 12);
        assert_eq!(unpacked.flag(), TTFlag::LowerBound);
        assert_eq!(unpacked.age(), 42);
    }
}
