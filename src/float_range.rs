pub struct FloatRange {
	current: f32,
	goal: f32,
}

impl Iterator for FloatRange {
	type Item = f32;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current < self.goal {
			let current = self.current.min(self.goal);
			self.current = self.current.floor() + 1.0;
			Some(current)
		} else if !self.current.is_nan() {
			self.current = f32::NAN;
			Some(self.goal)
		} else {
			None
		}
	}
}

pub fn range(from: f32, to: f32) -> FloatRange {
	FloatRange {
		current: from,
		goal: to,
	}
}
