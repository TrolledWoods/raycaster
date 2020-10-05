const WORLD: &[&[u8]] = &[
	b"###################",
	b"#........oooooooooo",
	b"#..#o#o##oooooooooo",
	b"#..o....ooooooooooo",
	b"#..#.#..ooooooooooo",
	b"#..o.#..#oooooooooo",
	b"#..#.#...oooooooooo",
	b"#..o.#..#oooooooooo",
	b"#..#.#...oooooooooo",
	b"#......##oooooooooo",
	b"#########oooooooooo",
];

pub struct World {
}

impl World {
	pub fn new() -> Self {
		World {}
	}

	#[inline]
	pub fn get(&self, x: isize, y: isize) -> Option<u8> {
		if x >= 0 && y >= 0 && (x as usize) < WORLD[0].len() && (y as usize) < WORLD.len() {
			Some(WORLD[y as usize][x as usize])
		} else {
			None
		}
	}
}
