pub struct World {
	tiles: Vec<Tile>,
	width: usize,
	height: usize,
}

impl World {
	pub fn new(world_string: &[&[u8]]) -> Self {
		let height = world_string.len();
		let width = world_string[0].len();

		let mut tiles = Vec::new();
		for &row in world_string {
			for tile in row {
				tiles.push(match tile {
					b'#' => Tile::Wall,
					b' ' => Tile::Floor,
					b'o' => Tile::Window,
					c => panic!("Unrecognised tile {}", c),
				});
			}
		}

		World {
			tiles,
			width,
			height,
		}
	}

	#[inline]
	pub fn get(&self, x: isize, y: isize) -> Option<&Tile> {
		if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
			Some(&self.tiles[y as usize * self.width + x as usize])
		} else {
			None
		}
	}
}

pub struct TileGraphics {
	pub texture: u16,
	pub is_transparent: bool,
}

pub enum Tile {
	Floor,
	Wall,
	Window,
}

impl Tile {
	pub fn get_graphics(&self) -> Option<TileGraphics> {
		match self {
			Tile::Floor  => None,
			Tile::Wall   => Some(TileGraphics { texture: 0, is_transparent: false }),
			Tile::Window => Some(TileGraphics { texture: 1, is_transparent: true }),
		}
	}
}
