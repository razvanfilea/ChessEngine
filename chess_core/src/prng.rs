pub struct Prng {
    seed: u64,
}

impl Default for Prng {
    fn default() -> Self {
        Self::new()
    }
}

impl Prng {
    pub const fn new() -> Self {
        Self { seed: 1070372 }
    }

    /*
     * Generate random numbers based on this paper: http://vigna.di.unimi.it/ftp/papers/xorshift.pdf
     */
    pub const fn random(&mut self) -> u64 {
        self.seed ^= self.seed >> 12;
        self.seed ^= self.seed << 25;
        self.seed ^= self.seed >> 27;

        self.seed.wrapping_mul(2685821657736338717)
    }

    pub const fn random_array<const N: usize>(&mut self) -> [u64; N] {
        let mut array = [0; N];
        let mut i = 0;
        while i < N {
            array[i] = self.random();
            i += 1;
        }
        array
    }

    pub const fn candidate(&mut self) -> u64 {
        self.random() & self.random() & self.random()
    }
}
