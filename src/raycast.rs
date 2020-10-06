use crate::Vec2;

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
			max_distance: 30.0,
		}
	}
}

pub fn raycast(
	cast: Raycast,
	mut data: impl FnMut(f32, isize, isize, f32, f32, Vec2) -> bool,
) {
	let mut total = 0.0;

	let mut ix = cast.x.floor() as isize;
	let mut iy = cast.y.floor() as isize;

	let mut x_remaining = if cast.dx.abs() < 0.000001 {
		100000000000000000.0f32
	} else if cast.dx > 0.0 {
		(1.0 - (cast.x - cast.x.floor())) / cast.dx
	} else {
		(cast.x - cast.x.floor()) / -cast.dx
	};

	let mut y_remaining = if cast.dy.abs() < 0.000001 {
		10000000000000000.0f32
	} else if cast.dy > 0.0 {
		(1.0 - (cast.y - cast.y.floor())) / cast.dy
	} else {
		(cast.y - cast.y.floor()) / -cast.dy
	};

	let mut pos = Vec2::new(cast.x, cast.y);

	while total < cast.max_distance {
		if x_remaining < y_remaining {
			total += x_remaining;
			pos += Vec2::new(cast.dx, cast.dy) * x_remaining;
			pos.x = pos.x.round();
			y_remaining -= x_remaining;
			x_remaining = 1.0 / cast.dx.abs().max(0.000001);
			ix += cast.dx.signum() as isize;
		} else {
			total += y_remaining;
			pos += Vec2::new(cast.dx, cast.dy) * y_remaining;
			pos.y = pos.y.round();
			x_remaining -= y_remaining;
			y_remaining = 1.0 / cast.dy.abs().max(0.000001);
			iy += cast.dy.signum() as isize;
		}

		if !data(
			total, ix, iy,
			pos.x.fract(), // (x_remaining * cast.dx + 1.0).fract(),
			pos.y.fract(), // (y_remaining * cast.dy + 1.0).fract(),
			pos
		) {
			break;
		}
	}
}
