use std::collections::HashMap;
use std::num::NonZeroU32;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(NonZeroU32);

pub struct World {
	pub tiles: TileMap,

	entities: HashMap<NonZeroU32, Entity>,
	entity_id_counter: NonZeroU32,
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
			tiles: TileMap {
				tiles,
				width,
				height,
			},

			entities: HashMap::new(),
			entity_id_counter: NonZeroU32::new(1).unwrap(),
		}
	}

	pub fn insert_entity(&mut self, entity: Entity) -> EntityId {
		let id = self.entity_id_counter;
		self.entity_id_counter = NonZeroU32::new(self.entity_id_counter.get() + 1).unwrap();

		let old = self.entities.insert(id, entity);
		assert!(old.is_none());

		EntityId(id)
	}

	pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
		self.entities.get(&id.0)
	}

	pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
		self.entities.get_mut(&id.0)
	}

	pub fn simulate_physics(&mut self, time_step: f32) {
		for entity in self.entities.values_mut() {
			entity.x += entity.vx * time_step;
			if self.tiles.square_is_colliding(entity.x, entity.y, entity.size) {
				entity.x -= entity.vx * time_step;
				entity.vx *= -0.6;
			}

			entity.y += entity.vy * time_step;
			if self.tiles.square_is_colliding(entity.x, entity.y, entity.size) {
				entity.y -= entity.vy * time_step;
				entity.vy *= -0.6;
			}

			entity.vx -= entity.vx * entity.move_drag * time_step;
			entity.vy -= entity.vy * entity.move_drag * time_step;
		}
	}
}

pub struct TileMap {
	tiles: Vec<Tile>,
	width: usize,
	height: usize,
}

impl TileMap {
	pub fn square_is_colliding(&self, x: f32, y: f32, size: f32) -> bool {
		let left = (x - size).floor() as isize;
		let right = (x + size).floor() as isize;
		let top = (y - size).floor() as isize;
		let bottom = (y + size).floor() as isize;

		for y in top..=bottom {
			for x in left..=right {
				if self.tile_is_colliding(x, y) {
					return true;
				}
			}
		}

		false
	}

	pub fn tile_is_colliding(&self, x: isize, y: isize) -> bool {
		if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
			self.tiles[y as usize * self.width + x as usize].is_solid()
		} else {
			false
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

	pub fn is_solid(&self) -> bool {
		match self {
			Tile::Floor => false,
			Tile::Wall => true,
			Tile::Window => true,
		}
	}
}

pub struct Entity {
	pub x: f32,
	pub y: f32,
	pub vx: f32,
	pub vy: f32,
	pub move_drag: f32,

	pub rot: f32,

	pub size: f32,
}

impl Entity {
	pub fn new(x: f32, y: f32, size: f32) -> Self {
		Entity {
			x,
			y,
			vx: 0.0,
			vy: 0.0,
			move_drag: 1.0,
			rot: 0.0,
			size,
		}
	}
}
