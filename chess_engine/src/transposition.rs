use std::sync::atomic::{AtomicU64, Ordering};

use crate::eval::MATE_THRESHOLD;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TTFlag {
    #[default]
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TTEntry {
    pub key: u32,
    pub value: i16,
    pub depth: u8,
    flag_age: u8,
}

impl TTEntry {
    const FLAG_MASK: u8 = 0b0011;

    pub fn get_age_flag(&self) -> (u8, TTFlag) {
        let flag = match self.flag_age & Self::FLAG_MASK {
            0 => TTFlag::Exact,
            1 => TTFlag::LowerBound,
            _ => TTFlag::UpperBound,
        };
        let age = self.flag_age >> 2;

        (age, flag)
    }

    #[inline(always)]
    pub fn get_flag(&self) -> TTFlag {
        self.get_age_flag().1
    }

    fn to_bits(self) -> u64 {
        (self.key as u64)
            | ((self.value as u16 as u64) << 32)
            | ((self.depth as u64) << 48)
            | ((self.flag_age as u64) << 56)
    }

    fn from_bits(bits: u64) -> Self {
        Self {
            key: bits as u32,
            value: (bits >> 32) as u16 as i16,
            depth: (bits >> 48) as u8,
            flag_age: (bits >> 56) as u8,
        }
    }
}
const BUCKET_SIZE: usize = 4;

pub struct TranspositionTable {
    /// `bucket_count * BUCKET_SIZE` slots; each bucket is a run of adjacent atomics.
    entries: Box<[AtomicU64]>,
    /// Current search generation, 6 bits (0..=63), matching `flag_age >> 2`.
    age: u8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let slots = (size_mb * 1024 * 1024) / std::mem::size_of::<AtomicU64>();
        let bucket_count = (slots / BUCKET_SIZE).max(1); // whole buckets only
        let entries = (0..bucket_count * BUCKET_SIZE)
            .map(|_| AtomicU64::new(0))
            .collect();
        Self { entries, age: 0 }
    }

    pub fn new_search(&mut self) {
        self.age = (self.age + 1) & 0x3F; // wraps at 64
    }

    pub fn clear(&self) {
        for slot in &self.entries {
            slot.store(0, Ordering::Relaxed);
        }
    }

    #[inline]
    fn bucket(&self, hash: u64) -> &[AtomicU64] {
        let bucket_count = (self.entries.len() / BUCKET_SIZE) as u64;
        // Lemire multiply-shift: derived from the HIGH bits of the hash.
        let idx = ((hash as u128 * bucket_count as u128) >> 64) as usize;
        let base = idx * BUCKET_SIZE;
        &self.entries[base..base + BUCKET_SIZE]
    }

    pub fn probe(&self, hash: u64, ply: u16) -> Option<TTEntry> {
        let key = hash as u32; // LOW bits, decorrelated from the index
        for slot in self.bucket(hash) {
            let mut entry = TTEntry::from_bits(slot.load(Ordering::Relaxed));
            if entry.key == key {
                entry.value = score_from_tt(entry.value, ply);
                return Some(entry);
            }
        }
        None
    }

    pub fn store(&self, hash: u64, value: i16, ply: u16, depth: u8, flag: TTFlag) {
        let key = hash as u32;
        let bucket = self.bucket(hash);

        // Pick the slot: an entry for this exact position wins; otherwise the
        // most replaceable one (an empty slot scores lowest, so it's chosen first).
        let mut victim = &bucket[0];
        let mut victim_entry = TTEntry::from_bits(victim.load(Ordering::Relaxed));
        let mut victim_quality = self.quality(&victim_entry);

        for slot in bucket {
            let entry = TTEntry::from_bits(slot.load(Ordering::Relaxed));
            if entry.key == key {
                victim = slot;
                victim_entry = entry;
                break;
            }
            let q = self.quality(&entry);
            if q < victim_quality {
                (victim, victim_entry, victim_quality) = (slot, entry, q);
            }
        }

        // Preserve a deeper entry for the SAME position unless the new result is
        // an exact bound or nearly as deep. A different position always overwrites.
        if victim_entry.key == key
            && flag != TTFlag::Exact
            && (depth as i16) + 2 < victim_entry.depth as i16
        {
            return;
        }

        let new = TTEntry {
            key,
            value: score_to_tt(value, ply),
            depth,
            flag_age: (self.age << 2) | flag as u8,
        };
        victim.store(new.to_bits(), Ordering::Relaxed);
    }

    /// Lower = more replaceable. Rewards depth, penalises stale generations.
    fn quality(&self, entry: &TTEntry) -> i32 {
        let (age, _) = entry.get_age_flag();
        let age_diff = (self.age.wrapping_sub(age) & 0x3F) as i32;
        entry.depth as i32 - age_diff * 2
    }
}

#[inline(always)]
fn score_to_tt(score: i16, ply: u16) -> i16 {
    if score > MATE_THRESHOLD {
        score + ply as i16
    } else if score < -MATE_THRESHOLD {
        score - ply as i16
    } else {
        score
    }
}

#[inline(always)]
fn score_from_tt(score: i16, ply: u16) -> i16 {
    if score > MATE_THRESHOLD {
        score - ply as i16
    } else if score < -MATE_THRESHOLD {
        score + ply as i16
    } else {
        score
    }
}
