#[derive(Clone)]
pub struct Random(u32);

impl Random {
	pub fn new() -> Self {
		use std::time::{SystemTime, UNIX_EPOCH};
		Random(
			(SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.expect("Cannot generate starting seed with epoch")
				.as_nanos() % 0xffff_ffff) as u32,
		)
	}

	pub fn with_seed(seed: u32) -> Self {
		Random(seed)
	}

	/// Generates a random 32 bit number
	pub fn get_32(&mut self) -> u32 {
		self.0 ^= self.0 << 13;
		self.0 ^= self.0 >> 17;
		self.0 ^= self.0 << 5;
		self.0
	}

	/// Generates a random floating point number between 0.0(inclusive) and 1.0(exclusive)
	pub fn get_float(&mut self) -> f32 {
		(self.get_32() & 0xffff) as f32 / 0x10000 as f32
	}
}
