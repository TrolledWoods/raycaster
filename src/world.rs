use std::collections::HashMap;
use std::num::NonZeroU32;

use crate::Vec2;

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
		let mut world = World {
			tiles: TileMap {
				width,
				height,

				tiles: vec![Tile::new(TileKind::Floor); width * height],
			},
			
			entities: HashMap::new(),
			entity_id_counter: NonZeroU32::new(1).unwrap(),
		};

		for (y, &row) in world_string.iter().enumerate() {
			for (x, tile) in row.iter().enumerate() {
				let tile = match tile {
					b'#' => Some(Tile::new(TileKind::Wall)),
					b' ' => None,
					b'o' => Some(Tile::new(TileKind::Window)),
					b'.' => {
						world.insert_entity(Entity {
							vel: Vec2::new(1.0, (x as f32 / 20.0).sin()),
							move_drag: 0.0,
							.. Entity::new(Vec2::new(x as f32 + 0.3, y as f32 + 0.3), 0.2, 2)
						});
						None
					}
					c => panic!("Unrecognised tile {}", c),
				};
				if let Some(tile) = tile {
					world.tiles.tiles[y * width + x] = tile;
				}
			}
		}

		world
	}

	pub fn insert_entity(&mut self, entity: Entity) -> EntityId {
		let id = self.entity_id_counter;
		self.entity_id_counter = NonZeroU32::new(self.entity_id_counter.get() + 1).unwrap();

		let texture_x = entity.pos.x;
		let texture_y = entity.pos.y;
		let texture_size = entity.texture_size;

		let old = self.entities.insert(id, entity);
		assert!(old.is_none());

		for y in (texture_y - texture_size / 2.0).floor() as isize ..=
				 (texture_y + texture_size / 2.0).floor() as isize {
			for x in (texture_x - texture_size / 2.0).floor() as isize ..=
					 (texture_x + texture_size / 2.0).floor() as isize {
				if let Some(tile) = self.tiles.get_mut(x, y) {
					tile.entities_inside.push(EntityId(id));
				}
			}
		}

		EntityId(id)
	}

	pub fn entities(&self) -> impl Iterator<Item = (EntityId, &Entity)> {
		self.entities.iter().map(|(&key, value)| (EntityId(key), value))
	}

	pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
		self.entities.get(&id.0)
	}

	pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
		self.entities.get_mut(&id.0)
	}

	pub fn simulate_physics(&mut self, time_step: f32) {
		for (entity_id, entity) in self.entities.iter_mut() {
			let mut new_pos = entity.pos;
			new_pos.x += entity.vel.x * time_step;
			if self.tiles.square_is_colliding(new_pos, entity.size) {
				new_pos.x -= entity.vel.x * time_step;
				entity.vel.x *= -1.0;
			}

			new_pos.y += entity.vel.y * time_step;
			if self.tiles.square_is_colliding(new_pos, entity.size) {
				new_pos.y -= entity.vel.y * time_step;
				entity.vel.y *= -1.0;
			}

			entity.vel -= entity.vel * entity.move_drag * time_step;

			self.tiles.move_entity(EntityId(*entity_id), entity, new_pos);
		}
	}

}

pub struct TileMap {
	tiles: Vec<Tile>,
	width: usize,
	height: usize,
}

impl TileMap {
	pub fn move_entity(&mut self, entity_id: EntityId, entity: &mut Entity, new_pos: Vec2) {
		let texture_size = entity.texture_size;

		let old_top    = (entity.pos.y - texture_size / 2.0).floor() as isize;
		let old_bottom = (entity.pos.y + texture_size / 2.0).floor() as isize;
		let old_left   = (entity.pos.x - texture_size / 2.0).floor() as isize;
		let old_right  = (entity.pos.x + texture_size / 2.0).floor() as isize;

		let new_top    = (new_pos.y - texture_size / 2.0).floor() as isize;
		let new_bottom = (new_pos.y + texture_size / 2.0).floor() as isize;
		let new_left   = (new_pos.x - texture_size / 2.0).floor() as isize;
		let new_right  = (new_pos.x + texture_size / 2.0).floor() as isize;

		entity.pos = new_pos;

		if old_top == new_top && old_bottom == new_bottom &&
			new_left == old_left && old_right == new_right
		{
			return;
		}

		for y in old_top ..= old_bottom {
			for x in old_left ..= old_right {
				if let Some(tile) = self.get_mut(x, y) {
					if let Some(loc) = tile.entities_inside.iter().position(|&v| v == entity_id) {
						tile.entities_inside.swap_remove(loc);
					}
				}
			}
		}

		for y in new_top ..= new_bottom {
			for x in new_left ..= new_right {
				if let Some(tile) = self.get_mut(x, y) {
					tile.entities_inside.push(entity_id);
				}
			}
		}
	}

	pub fn square_is_colliding(&self, pos: Vec2, size: f32) -> bool {
		let left = (pos.x - size).floor() as isize;
		let right = (pos.x + size).floor() as isize;
		let top = (pos.y - size).floor() as isize;
		let bottom = (pos.y + size).floor() as isize;

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

	#[inline]
	pub fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut Tile> {
		if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
			Some(&mut self.tiles[y as usize * self.width + x as usize])
		} else {
			None
		}
	}
}

pub struct TileGraphics {
	pub texture: u16,
	pub is_transparent: bool,
}

#[derive(Clone)]
pub struct Tile {
	pub kind: TileKind,
	pub entities_inside: Vec<EntityId>,
}

#[derive(Clone)]
pub enum TileKind {
	Floor,
	Wall,
	Window,
}

impl Tile {
	pub fn new(kind: TileKind) -> Self {
		Tile {
			kind,
			entities_inside: Vec::new(),
		}
	}

	pub fn get_graphics(&self) -> Option<TileGraphics> {
		match self.kind {
			TileKind::Floor => None,
			TileKind::Wall   => Some(TileGraphics { texture: 0, is_transparent: false }),
			TileKind::Window => Some(TileGraphics { texture: 1, is_transparent: true }),
		}
	}

	pub fn is_solid(&self) -> bool {
		match self.kind {
			TileKind::Floor => false,
			TileKind::Wall => true,
			TileKind::Window => true,
		}
	}
}

pub struct Entity {
	pub pos: Vec2,
	pub vel: Vec2,
	pub move_drag: f32,
	pub rot: f32,
	pub size: f32,
	pub texture: u16,
	pub texture_size: f32,
}

impl Entity {
	pub fn new(pos: Vec2, size: f32, texture: u16) -> Self {
		Entity {
			pos,
			vel: Vec2::zero(),
			move_drag: 1.0,
			rot: 0.0,
			size,
			texture,
			texture_size: 0.5,
		}
	}
}
