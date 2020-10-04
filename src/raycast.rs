#[derive(Clone, Copy, Debug)]
pub struct Raycast {
	pub x: f32,
	pub y: f32,
	pub dx: f32,
	pub dy: f32,
	pub max_distance: f32,
}

impl Default for Raycast {
	fn default() -> Self {
		Raycast {
			x: 0.0,
			y: 0.0,
			dx: 0.0,
			dy: 0.0,
			max_distance: 100.0,
		}
	}
}

pub fn raycast<Hit>(
	mut cast: Raycast,
	mut data: impl FnMut(isize, isize) -> Option<Hit>,
) -> Option<(f32, Hit)> {
	let mut distance = 0.0;
	let step_size = 0.1;

	while distance < cast.max_distance {
		if let Some(hit) = data(cast.x.floor() as isize, cast.y.floor() as isize) {
			return Some((distance, hit));
		}

		cast.x += cast.dx * step_size;
		cast.y += cast.dy * step_size;
		distance += step_size;
	}

	None
}
