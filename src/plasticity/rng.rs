//! A small seeded generator (`SplitMix64`): the same seed always gives the same
//! exercise, which is what lets an answer be checked later.

pub struct Rng(u64);

impl Rng {
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub const fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// `0..n`.
    pub const fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    /// `lo..=hi`.
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    pub const fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + self.below((hi - lo + 1) as u64) as i64
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len() as u64) as usize]
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn shuffle<T>(&mut self, xs: &mut [T]) {
        for i in (1..xs.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            xs.swap(i, j);
        }
    }

    /// True one time in `n`.
    pub const fn one_in(&mut self, n: u64) -> bool {
        self.below(n) == 0
    }
}
