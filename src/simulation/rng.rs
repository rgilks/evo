//! Seeding and RNG helpers for the simulation.
//!
//! All randomness is keyed off a single `u64` seed so a run reproduces exactly
//! (see ARCHITECTURE). `mix_seed` derives a well-distributed per-entity/per-tick
//! stream seed; `generate_particle_matrix` builds the per-seed interaction matrix.

use rand::{Error, Rng, RngCore, SeedableRng};

/// Default seed used by the native/test path so runs are reproducible. The
/// browser seeds from the wall clock for per-load variety (see lib.rs).
pub const DEFAULT_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// Golden-ratio gamma constant for the SplitMix64 stream step.
const SPLITMIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// A tiny, fast, seedable PRNG (SplitMix64) used for all of the simulation's
/// hot-loop randomness. State is a single `u64`: each draw advances by the
/// golden-ratio gamma and runs a splitmix64 finaliser, which is well enough
/// distributed for `gen_range` / `gen::<f32>()` while being far cheaper than
/// `StdRng`. Seeding is deterministic, so a run reproduces exactly from its seed.
#[derive(Clone)]
pub(crate) struct FastRng {
    state: u64,
}

impl FastRng {
    /// Advance the stream and return the next 64-bit output.
    #[inline]
    fn next_split(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX_GAMMA);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

impl SeedableRng for FastRng {
    type Seed = [u8; 8];

    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            state: u64::from_le_bytes(seed),
        }
    }

    /// Seed directly from a `u64`, matching the `StdRng::seed_from_u64` call
    /// sites that drive every RNG stream off `mix_seed(...)`.
    fn seed_from_u64(state: u64) -> Self {
        Self { state }
    }
}

impl RngCore for FastRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        // High bits of splitmix64 are its best-mixed, so take the top 32.
        (self.next_split() >> 32) as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.next_split()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut chunks = dest.chunks_exact_mut(8);
        for chunk in &mut chunks {
            chunk.copy_from_slice(&self.next_split().to_le_bytes());
        }
        let rem = chunks.into_remainder();
        if !rem.is_empty() {
            let bytes = self.next_split().to_le_bytes();
            rem.copy_from_slice(&bytes[..rem.len()]);
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

/// Salt mixed into offspring RNG so a parent's reproduction stream is distinct
/// from its movement stream within the same tick.
pub const OFFSPRING_SALT: u64 = 0x0FF5_0FF5_0FF5_0FF5;

/// Salts for the user "cull"/"bloom" action RNG streams.
pub const CULL_SALT: u64 = 0xCADE_1234_5678_9ABC;
pub const BLOOM_SALT: u64 = 0xB100_4321_8765_CBA9;

/// Salt for the per-seed particle-life interaction matrix RNG stream.
const MATRIX_SALT: u64 = 0xC0FF_EE15_600D_F00D;

/// Mix a base seed with an entity id and tick into a well-distributed seed
/// (splitmix64 finaliser). Keying RNG per entity+tick makes results independent
/// of thread scheduling, so the simulation is reproducible from a single seed.
pub fn mix_seed(seed: u64, entity_bits: u64, step: u64) -> u64 {
    let mut x = seed
        ^ entity_bits.wrapping_mul(0xD1B5_4A32_D192_ED03)
        ^ step.wrapping_mul(0x94D0_49BB_1331_11EB);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Build the global particle-life interaction matrix from the seed. Every creature
/// shares it, indexed by `[self hue-sector][other hue-sector]`, so colour groups
/// attract or repel each other *coherently* — that coherence is what makes clusters
/// form and move as units instead of dissolving into a uniform haze. Per-seed, so
/// each seed is a different "physics" (and shareable via the seed).
pub fn generate_particle_matrix(seed: u64) -> [[f32; 6]; 6] {
    let mut rng = FastRng::seed_from_u64(mix_seed(seed, MATRIX_SALT, 0));
    let mut matrix = [[0.0f32; 6]; 6];
    for row in matrix.iter_mut() {
        for weight in row.iter_mut() {
            *weight = rng.gen_range(-1.0..1.0);
        }
    }
    matrix
}
